use crate::{Canvas, Color, RectI32};
use fontdue::Font;
use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use watchcompare_project::{Card, ModelKind, Project, RgbaColor};
use watchcompare_scene::{sample_reference_scene, BadgeSceneState, ReferenceSceneState};

const REF_WIDTH: u32 = 1920;
const REF_HEIGHT: u32 = 1080;
const REF_MAIN_PITCH: i32 = 477;
const REF_MAIN_BODY: u32 = 471;
const REF_INTRO_PITCH: i32 = 480;
const REF_INTRO_BODY: u32 = 475;
const REF_INTRO_CARD_START: [u64; 4] = [5, 125, 244, 363];
const REF_PAN_START: u64 = 524;
const REF_OUTRO_START: u64 = 11_843;
const REF_CREDITS_RETRACT: u64 = 396;
const REF_CREDITS_GONE: u64 = 429;

fn color(value: RgbaColor) -> Color {
    Color::rgba(value.0, value.1, value.2, value.3)
}

fn measure_text(font: &Font, text: &str, px: f32) -> f32 {
    text.chars().map(|ch| font.metrics(ch, px).advance_width).sum::<f32>()
}

fn draw_centered_text(
    canvas: &mut Canvas,
    font: &Font,
    text: &str,
    px: f32,
    center_x: f32,
    baseline_y: f32,
    ink: Color,
) {
    let width = measure_text(font, text, px);
    canvas.draw_text_baseline(font, text, px, center_x - width / 2.0, baseline_y, ink);
}

fn wrap_lines(font: &Font, text: &str, px: f32, max_width: f32, max_lines: usize) -> Vec<String> {
    if text.trim().is_empty() || max_lines == 0 {
        return Vec::new();
    }
    let words = text.split_whitespace().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in words {
        let candidate = if current.is_empty() { word.to_string() } else { format!("{current} {word}") };
        if !current.is_empty() && measure_text(font, &candidate, px) > max_width {
            lines.push(current);
            current = word.to_string();
            if lines.len() + 1 >= max_lines {
                break;
            }
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    lines
}

fn draw_centered_multiline(
    canvas: &mut Canvas,
    font: &Font,
    text: &str,
    px: f32,
    center_x: f32,
    top: f32,
    bottom: f32,
    max_width: f32,
    max_lines: usize,
    ink: Color,
) {
    let lines = wrap_lines(font, text, px, max_width, max_lines);
    if lines.is_empty() {
        return;
    }
    let line_height = px * 0.88;
    let block_height = line_height * lines.len() as f32;
    let mut baseline = (top + bottom - block_height) * 0.5 + px * 0.76;
    for line in lines {
        draw_centered_text(canvas, font, &line, px, center_x, baseline, ink);
        baseline += line_height;
    }
}

fn draw_fallback_word(canvas: &mut Canvas, text: &str, x: i32, y: i32, max_width: u32, ink: Color) {
    if text.is_empty() || max_width < 8 {
        return;
    }
    let chars = text.chars().take(42).collect::<Vec<_>>();
    let cell = ((max_width as usize / chars.len().max(1)).clamp(4, 11)) as i32;
    let glyph_w = (cell - 2).max(2);
    for (i, ch) in chars.into_iter().enumerate() {
        if ch.is_whitespace() {
            continue;
        }
        let seed = ch as u32;
        let gx = x + i as i32 * cell;
        for row in 0..7_i32 {
            let bits = seed.rotate_left(row as u32) ^ seed.wrapping_mul(0x45d9f3b);
            for col in 0..5_i32 {
                if ((bits >> col) & 1) == 0 {
                    continue;
                }
                let px = gx + col * glyph_w / 5;
                canvas.fill_rect(
                    RectI32 {
                        x: px,
                        y: y + row * 2,
                        width: (glyph_w / 5).max(1) as u32,
                        height: 2,
                    },
                    ink,
                );
            }
        }
    }
}

fn draw_artwork(canvas: &mut Canvas, card: &Card, x: i32, width: u32, height: u32) {
    canvas.fill_rect(RectI32 { x, y: 0, width, height }, color(card.artwork_color));
    let Some(path) = card.artwork_path.as_deref() else {
        return;
    };
    let Ok(image) = image::open(path) else {
        return;
    };
    let rgba = image.to_rgba8();
    if rgba.width() == 0 || rgba.height() == 0 {
        return;
    }
    let scale = (width as f64 / rgba.width() as f64).max(height as f64 / rgba.height() as f64);
    let rw = (rgba.width() as f64 * scale).ceil().max(1.0) as u32;
    let rh = (rgba.height() as f64 * scale).ceil().max(1.0) as u32;
    let resized = image::imageops::resize(&rgba, rw, rh, FilterType::Lanczos3);
    let crop_x = resized.width().saturating_sub(width) / 2;
    let crop_y = resized.height().saturating_sub(height) / 2;
    let cropped = image::imageops::crop_imm(
        &resized,
        crop_x,
        crop_y,
        width.min(resized.width()),
        height.min(resized.height()),
    )
    .to_image();
    canvas.blit(&cropped, x, 0);
}

fn point_in_polygon(x: f32, y: f32, vertices: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let (xi, yi) = vertices[i];
        let (xj, yj) = vertices[j];
        let crosses = ((yi > y) != (yj > y))
            && (x < (xj - xi) * (y - yi) / ((yj - yi).abs().max(f32::EPSILON)) + xi);
        if crosses {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn draw_masked_shine(
    canvas: &mut Canvas,
    vertices: &[(f32, f32)],
    axis_degrees: f32,
    center_normal_px: f32,
    half_width_px: f32,
    feather_px: f32,
) {
    if vertices.len() < 3 {
        return;
    }
    let radians = axis_degrees.to_radians();
    let nx = -radians.sin();
    let ny = radians.cos();
    let min_x = vertices.iter().map(|v| v.0).fold(f32::INFINITY, f32::min).floor() as i32;
    let max_x = vertices.iter().map(|v| v.0).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
    let min_y = vertices.iter().map(|v| v.1).fold(f32::INFINITY, f32::min).floor() as i32;
    let max_y = vertices.iter().map(|v| v.1).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
    let feather = feather_px.max(0.5);
    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let cx = px as f32 + 0.5;
            let cy = py as f32 + 0.5;
            if !point_in_polygon(cx, cy, vertices) {
                continue;
            }
            let d = ((cx * nx + cy * ny) - center_normal_px).abs();
            if d > half_width_px + feather {
                continue;
            }
            let coverage = if d <= half_width_px {
                1.0
            } else {
                1.0 - (d - half_width_px) / feather
            };
            canvas.blend_pixel(px, py, Color::rgba(255, 255, 255, (142.0 * coverage) as u8));
        }
    }
}

#[derive(Clone, Copy)]
struct BadgeVisual {
    visible: bool,
    scale: f32,
    text_alpha: u8,
    shine: Option<watchcompare_render::ShineSample>,
}

fn badge_visual_from_scene(state: BadgeSceneState) -> BadgeVisual {
    if !state.visible {
        return BadgeVisual { visible: false, scale: 1.0, text_alpha: 0, shine: None };
    }
    let scale = state
        .transform
        .map(|v| v.scale)
        .or_else(|| state.visible_core_bbox.map(|bbox| bbox.width as f32 / 283.0))
        .unwrap_or(1.0)
        .clamp(0.04, 1.65);
    BadgeVisual {
        visible: true,
        scale,
        text_alpha: (state.text_reveal_level.clamp(0.0, 1.0) * 255.0).round() as u8,
        shine: state.shine,
    }
}

fn reference_badge_visual(frame: u64, card_index: usize) -> BadgeVisual {
    let template_frame = match card_index {
        0 => frame.checked_add(116),
        1 => Some(frame),
        2 => frame.checked_sub(119),
        3 => frame.checked_sub(238),
        _ => None,
    };
    if let Some(template_frame) = template_frame {
        return badge_visual_from_scene(sample_reference_scene(template_frame).second_badge);
    }
    BadgeVisual { visible: true, scale: 1.0, text_alpha: 255, shine: None }
}

fn draw_badge(
    canvas: &mut Canvas,
    card: &Card,
    font: Option<&Font>,
    x: i32,
    width: u32,
    badge_color: Color,
    badge_px: f32,
    reference: Option<BadgeVisual>,
) {
    if card.badge_value.is_empty() && card.badge_label.is_empty() {
        return;
    }

    let visual = reference.unwrap_or(BadgeVisual {
        visible: true,
        scale: 1.0,
        text_alpha: 255,
        shine: None,
    });
    if !visual.visible {
        return;
    }

    let (settled_w, settled_top) = if reference.is_some() { (246.0_f32, 68.0_f32) } else { (298.0_f32.min(width as f32 - 32.0).max(120.0), 20.0_f32) };
    let badge_w = (settled_w * visual.scale).max(4.0);
    let source_scale = badge_w / 298.0;
    let badge_h = 344.0 * source_scale;
    let settled_h = 344.0 * (settled_w / 298.0);
    let left = x as f32 + (width as f32 - badge_w) / 2.0;
    let top = if reference.is_some() {
        settled_top + (settled_h - badge_h) / 2.0
    } else {
        settled_top
    };
    let vertices = [
        (148.0, 0.0),
        (2.0, 84.0),
        (0.0, 255.0),
        (151.0, 343.0),
        (297.0, 257.0),
        (297.0, 84.0),
    ]
    .into_iter()
    .map(|(vx, vy)| (left + vx * source_scale, top + vy * source_scale))
    .collect::<Vec<_>>();
    canvas.fill_polygon(&vertices, badge_color);

    if let Some(shine) = visual.shine {
        let axis = 120.0_f32;
        let radians = axis.to_radians();
        let nx = -radians.sin();
        let ny = radians.cos();
        let global_center = left * nx + top * ny + shine.normal_center_px * source_scale;
        draw_masked_shine(
            canvas,
            &vertices,
            axis,
            global_center,
            shine.width80_px * source_scale * 0.5,
            (9.0 * source_scale).max(2.0),
        );
    }

    let center = left + badge_w / 2.0;
    if let Some(font) = font {
        let ink = Color::rgba(255, 255, 255, visual.text_alpha);
        draw_centered_text(
            canvas,
            font,
            &card.badge_value,
            badge_px * source_scale.max(0.75),
            center,
            top + badge_h * 0.46,
            ink,
        );
        draw_centered_text(
            canvas,
            font,
            &card.badge_label,
            badge_px * 0.42 * source_scale.max(0.75),
            center,
            top + badge_h * 0.60,
            ink,
        );
    } else if visual.text_alpha > 24 {
        draw_fallback_word(
            canvas,
            &card.badge_value,
            (left + 38.0 * source_scale) as i32,
            (top + badge_h * 0.35) as i32,
            (badge_w - 76.0 * source_scale).max(1.0) as u32,
            Color::WHITE,
        );
    }
}

fn draw_card_with_geometry(
    canvas: &mut Canvas,
    project: &Project,
    card: &Card,
    x: i32,
    font: Option<&Font>,
    pitch: u32,
    separator: u32,
    reference_badge: Option<BadgeVisual>,
) {
    let theme = &project.theme;
    let card_width = pitch.saturating_sub(separator).max(1);
    let scale_y = project.export.height as f64 / 1080.0;
    let sy = |value: u32| (value as f64 * scale_y).round() as u32;
    let art_h = sy(theme.artwork_bottom_y.saturating_add(1)).min(project.export.height);
    let title_top = sy(theme.title_top_y).min(project.export.height);
    let title_bottom = sy(theme.title_bottom_y.saturating_add(1)).min(project.export.height);
    let desc_top = sy(theme.description_top_y).min(project.export.height);
    let desc_bottom = sy(theme.description_bottom_y.saturating_add(1)).min(project.export.height);

    draw_artwork(canvas, card, x, card_width, art_h);
    if title_bottom > title_top {
        let panel = if project.model == ModelKind::ReferenceLocked {
            Color::rgba(242, 242, 242, 255)
        } else {
            color(card.title_panel_color)
        };
        canvas.fill_rect(
            RectI32 { x, y: title_top as i32, width: card_width, height: title_bottom - title_top },
            panel,
        );
    }
    if desc_bottom > desc_top {
        let panel = if project.model == ModelKind::ReferenceLocked {
            Color::rgba(96, 96, 91, 255)
        } else {
            color(card.description_panel_color)
        };
        canvas.fill_rect(
            RectI32 { x, y: desc_top as i32, width: card_width, height: desc_bottom - desc_top },
            panel,
        );
    }

    draw_badge(
        canvas,
        card,
        font,
        x,
        card_width,
        color(theme.badge_color),
        theme.badge_font_px,
        reference_badge,
    );

    let center = x as f32 + card_width as f32 / 2.0;
    if let Some(font) = font {
        draw_centered_multiline(
            canvas,
            font,
            &card.title,
            theme.title_font_px * scale_y as f32,
            center,
            title_top as f32,
            title_bottom as f32,
            card_width.saturating_sub(24) as f32,
            2,
            color(theme.title_text_color),
        );
        if !card.description.is_empty() {
            draw_centered_multiline(
                canvas,
                font,
                &card.description,
                theme.description_font_px * scale_y as f32,
                center,
                desc_top as f32,
                desc_bottom as f32,
                card_width.saturating_sub(28) as f32,
                2,
                color(theme.description_text_color),
            );
        }
    } else {
        draw_fallback_word(
            canvas,
            &card.title,
            x + 20,
            title_top as i32 + 22,
            card_width.saturating_sub(40),
            color(theme.title_text_color),
        );
        draw_fallback_word(
            canvas,
            &card.description,
            x + 20,
            desc_top as i32 + 28,
            card_width.saturating_sub(40),
            color(theme.description_text_color),
        );
    }
}

fn draw_card_clipped(
    canvas: &mut Canvas,
    project: &Project,
    card: &Card,
    x: i32,
    font: Option<&Font>,
    pitch: u32,
    separator: u32,
    visible_width: u32,
    reference_badge: Option<BadgeVisual>,
) {
    if visible_width == 0 {
        return;
    }
    let body = pitch.saturating_sub(separator).max(1);
    if visible_width >= body {
        draw_card_with_geometry(canvas, project, card, x, font, pitch, separator, reference_badge);
        return;
    }
    let mut temp = Canvas::new(body, project.export.height, Color::TRANSPARENT);
    draw_card_with_geometry(&mut temp, project, card, 0, font, pitch, separator, reference_badge);
    let width = visible_width.min(body);
    let cropped = image::imageops::crop_imm(temp.as_image(), 0, 0, width, project.export.height).to_image();
    canvas.blit(&cropped, x, 0);
}

fn reference_reveal_width(frame: u64, card_index: usize) -> u32 {
    let Some(start) = REF_INTRO_CARD_START.get(card_index).copied() else {
        return REF_INTRO_BODY;
    };
    if frame < start {
        return 0;
    }
    let template = 5 + (frame - start);
    sample_reference_scene(template).first_card_reveal_width_px as u32
}

fn draw_reference_credits(canvas: &mut Canvas, font: Option<&Font>, scene: &ReferenceSceneState) {
    if scene.frame >= REF_CREDITS_GONE {
        return;
    }
    let left = if scene.frame < REF_CREDITS_RETRACT {
        1440
    } else {
        scene.credits_left_x_px.unwrap_or(1920) as i32
    };
    if left >= REF_WIDTH as i32 {
        return;
    }
    canvas.fill_rect(
        RectI32 { x: left, y: 0, width: REF_WIDTH.saturating_sub(left.max(0) as u32), height: REF_HEIGHT },
        Color::rgba(16, 16, 16, 255),
    );

    let Some(font) = font else {
        return;
    };
    let center = (left as f32 + REF_WIDTH as f32) * 0.5;
    let white = Color::WHITE;
    draw_centered_multiline(
        canvas,
        font,
        "The values presented are the years in which various developments resulted in languages occurred",
        16.0,
        center,
        45.0,
        145.0,
        390.0,
        4,
        white,
    );
    canvas.fill_rect(
        RectI32 { x: left + 82, y: 183, width: (REF_WIDTH as i32 - left - 164).max(0) as u32, height: 1 },
        Color::rgba(96, 96, 96, 255),
    );
    draw_centered_text(canvas, font, "Credits", 34.0, center, 244.0, white);
    let lines = [
        "Lead Research & Sourcing — Ahmed",
        "Independent Fact Check — Alex Lambert",
        "Lead Graphic Designer — Jack H",
        "Edit & Post-Production — Alex Pacheco",
        "Thumbnail Designer — Diego Garcia",
        "Video Idea & Quality Check — Ideaguy.co",
    ];
    let mut y = 300.0;
    for line in lines {
        draw_centered_text(canvas, font, line, 14.0, center, y, white);
        y += 58.0;
    }
}

fn draw_mid_video_cta(canvas: &mut Canvas, scene: &ReferenceSceneState) {
    if !scene.mid_video_cta.visible {
        return;
    }
    if let Some(rect) = scene.mid_video_cta.red_bbox {
        canvas.fill_rect(
            RectI32 {
                x: rect.x as i32,
                y: rect.y as i32,
                width: rect.width as u32,
                height: rect.height as u32,
            },
            Color::rgba(223, 16, 36, 255),
        );
    }
}

fn draw_cursor(canvas: &mut Canvas, x: f32, y: f32) {
    let x = x.round();
    let y = y.round();
    let outer = [(x, y), (x + 7.0, y + 22.0), (x + 11.0, y + 15.0), (x + 19.0, y + 15.0)];
    canvas.fill_polygon(&outer, Color::BLACK);
    let inner = [(x + 1.5, y + 2.0), (x + 7.0, y + 19.0), (x + 10.0, y + 13.0), (x + 16.0, y + 13.0)];
    canvas.fill_polygon(&inner, Color::WHITE);
}

fn draw_reference_outro(canvas: &mut Canvas, project: &Project, font: Option<&Font>, scene: &ReferenceSceneState) {
    let left_width = (REF_WIDTH as i32 - REF_MAIN_PITCH).max(0) as u32;
    if let Some(bottom) = scene.outro.wipe_bottom_y {
        canvas.fill_rect(
            RectI32 { x: 0, y: 0, width: left_width, height: (bottom as u32 + 1).min(REF_HEIGHT) },
            Color::BLACK,
        );
    }

    if let Some(group) = scene.outro.group {
        let y = group.panel_top_y as i32;
        canvas.fill_rect(RectI32 { x: 55, y, width: 610, height: 330 }, Color::rgba(206, 13, 31, 255));
        canvas.fill_rect(RectI32 { x: 770, y, width: 610, height: 330 }, Color::rgba(206, 13, 31, 255));
        if let Some(font) = font {
            draw_centered_text(canvas, font, "BEST VIDEO FOR YOU", 25.0, 360.0, y as f32 + 300.0, Color::WHITE);
            draw_centered_text(canvas, font, "NEWEST VIDEO", 25.0, 1075.0, y as f32 + 300.0, Color::WHITE);
            draw_centered_text(canvas, font, "Video Made By", 34.0, 720.0, group.credits_top_y as f32, Color::rgba(145, 145, 145, 255));
        }
    }

    if let Some(bbox) = scene.cta.outer_bbox {
        canvas.fill_rect(
            RectI32 { x: bbox.x as i32, y: bbox.y as i32, width: bbox.width as u32, height: bbox.height as u32 },
            Color::WHITE,
        );
        if let Some(font) = font {
            let center_y = bbox.y as f32 + bbox.height as f32 * 0.66;
            let label = if scene.cta.subscribed { "SUBSCRIBED" } else { "SUBSCRIBE" };
            draw_centered_text(
                canvas,
                font,
                label,
                26.0,
                bbox.x as f32 + bbox.width as f32 * 0.56,
                center_y,
                Color::rgba(35, 35, 35, 255),
            );
        }
    }

    if let (Some(x), Some(y)) = (scene.cta.cursor_x_px, scene.cta.cursor_y_px) {
        draw_cursor(canvas, x, y);
    }

    if scene.outro.fade_level < 1.0 {
        let alpha = ((1.0 - scene.outro.fade_level).clamp(0.0, 1.0) * 255.0).round() as u8;
        canvas.fill_rect(
            RectI32 { x: 0, y: 0, width: project.export.width, height: project.export.height },
            Color::rgba(0, 0, 0, alpha),
        );
    }
}

fn reference_outro_card_index(project: &Project, scene: &ReferenceSceneState) -> usize {
    let pitch = REF_MAIN_PITCH as f64;
    let rightmost_virtual = ((REF_WIDTH as f64 - scene.card_train_x_px) / pitch).ceil().max(1.0) as usize - 1;
    rightmost_virtual % project.cards.len()
}

fn render_reference_locked(project: &Project, frame: u64, font: Option<&Font>) -> Result<RgbaImage, String> {
    let scene = sample_reference_scene(frame);
    let mut canvas = Canvas::new(REF_WIDTH, REF_HEIGHT, Color::rgba(16, 16, 16, 255));

    if scene.frame < REF_PAN_START {
        for index in 0..4_usize {
            let card = &project.cards[index % project.cards.len()];
            let visible = reference_reveal_width(scene.frame, index).min(REF_INTRO_BODY);
            let x = index as i32 * REF_INTRO_PITCH;
            draw_card_clipped(
                &mut canvas,
                project,
                card,
                x,
                font,
                REF_INTRO_PITCH as u32,
                (REF_INTRO_PITCH as u32).saturating_sub(REF_INTRO_BODY),
                visible,
                Some(reference_badge_visual(scene.frame, index)),
            );
        }
        draw_reference_credits(&mut canvas, font, &scene);
    } else if scene.frame < REF_OUTRO_START {
        let first_virtual = ((-scene.card_train_x_px / REF_MAIN_PITCH as f64).floor() as i64 - 1).max(0) as usize;
        let last_virtual = (((REF_WIDTH as f64 - scene.card_train_x_px) / REF_MAIN_PITCH as f64).ceil() as usize + 1).max(first_virtual);
        for virtual_index in first_virtual..=last_virtual {
            let x = (scene.card_train_x_px + virtual_index as f64 * REF_MAIN_PITCH as f64).round() as i32;
            if x + REF_MAIN_PITCH < 0 || x >= REF_WIDTH as i32 {
                continue;
            }
            let card = &project.cards[virtual_index % project.cards.len()];
            draw_card_with_geometry(
                &mut canvas,
                project,
                card,
                x,
                font,
                REF_MAIN_PITCH as u32,
                (REF_MAIN_PITCH as u32).saturating_sub(REF_MAIN_BODY),
                if virtual_index < 4 { Some(reference_badge_visual(scene.frame, virtual_index)) } else { None },
            );
        }
    } else {
        let card_index = reference_outro_card_index(project, &scene);
        let card = &project.cards[card_index];
        draw_card_with_geometry(
            &mut canvas,
            project,
            card,
            REF_WIDTH as i32 - REF_MAIN_PITCH,
            font,
            REF_MAIN_PITCH as u32,
            (REF_MAIN_PITCH as u32).saturating_sub(REF_MAIN_BODY),
            None,
        );
        draw_reference_outro(&mut canvas, project, font, &scene);
    }

    draw_mid_video_cta(&mut canvas, &scene);
    Ok(canvas.into_image())
}

fn render_generic(project: &Project, frame: u64, font: Option<&Font>) -> Result<RgbaImage, String> {
    let mut canvas = Canvas::new(project.export.width, project.export.height, color(project.theme.background_color));
    let fps = project.export.fps.max(1) as f64;
    let pitch = project.theme.card_pitch_px;
    let elapsed = frame as f64 / fps;
    let motion = elapsed * project.theme.scroll_px_per_second;
    let start_x = project.export.width as f64 * 0.5 - pitch * 0.5;
    for (index, card) in project.cards.iter().enumerate() {
        let x = start_x + index as f64 * pitch - motion;
        if x + pitch < 0.0 || x > project.export.width as f64 {
            continue;
        }
        draw_card_with_geometry(
            &mut canvas,
            project,
            card,
            x.round() as i32,
            font,
            pitch.round().max(2.0) as u32,
            project.theme.separator_px,
            None,
        );
    }
    Ok(canvas.into_image())
}

/// Render one full project frame. Preview and export call this exact same Rust function.
pub fn render_project_frame(project: &Project, frame: u64, font: Option<&Font>) -> Result<RgbaImage, String> {
    project.validate()?;
    if project.model == ModelKind::ReferenceLocked {
        render_reference_locked(project, frame, font)
    } else {
        render_generic(project, frame, font)
    }
}

pub fn render_project_png(project: &Project, frame: u64, font: Option<&Font>) -> Result<Vec<u8>, String> {
    let image = render_project_frame(project, frame, font)?;
    let mut bytes = Vec::new();
    DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|e| format!("encode png: {e}"))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_renderer_produces_requested_size() {
        let mut project = Project::demo();
        project.model = ModelKind::Clean;
        project.export.width = 640;
        project.export.height = 360;
        let image = render_project_frame(&project, 0, None).unwrap();
        assert_eq!(image.dimensions(), (640, 360));
    }

    #[test]
    fn reference_locked_stays_on_measured_raster() {
        let project = Project::demo();
        let image = render_project_frame(&project, 396, None).unwrap();
        assert_eq!(image.dimensions(), (1920, 1080));
    }

    #[test]
    fn preview_png_is_valid_png() {
        let mut project = Project::demo();
        project.model = ModelKind::Clean;
        project.export.width = 320;
        project.export.height = 180;
        let png = render_project_png(&project, 12, None).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn reference_card_train_uses_scene_trace_after_intro() {
        let project = Project::demo();
        let scene = sample_reference_scene(630);
        assert!(scene.card_train_x_px < 0.0);
        let image = render_project_frame(&project, 630, None).unwrap();
        assert_eq!(image.width(), 1920);
    }
}
