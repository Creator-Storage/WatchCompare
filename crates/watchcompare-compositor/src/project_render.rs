use crate::{Canvas, Color, RectI32};
use fontdue::Font;
use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use watchcompare_project::{Card, Project, RgbaColor};

fn color(value: RgbaColor) -> Color {
    Color::rgba(value.0, value.1, value.2, value.3)
}

fn measure_text(font: &Font, text: &str, px: f32) -> f32 {
    text.chars()
        .map(|ch| font.metrics(ch, px).advance_width)
        .sum::<f32>()
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

fn draw_fallback_word(canvas: &mut Canvas, text: &str, x: i32, y: i32, max_width: u32, ink: Color) {
    if text.is_empty() || max_width < 8 {
        return;
    }
    // Deterministic no-font fallback. It is intentionally simple; exact reference
    // exports should provide the locally-owned Pin Sans font path.
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
            let bits = seed.rotate_left(row as u32) ^ (seed.wrapping_mul(0x45d9f3b));
            for col in 0..5_i32 {
                if ((bits >> col) & 1) == 0 {
                    continue;
                }
                let px = gx + col * glyph_w / 5;
                canvas.fill_rect(
                    RectI32 { x: px, y: y + row * 2, width: (glyph_w / 5).max(1) as u32, height: 2 },
                    ink,
                );
            }
        }
    }
}

fn draw_artwork(canvas: &mut Canvas, card: &Card, x: i32, width: u32, height: u32) {
    canvas.fill_rect(RectI32 { x, y: 0, width, height }, color(card.artwork_color));
    let Some(path) = card.artwork_path.as_deref() else { return; };
    let Ok(image) = image::open(path) else { return; };
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
    let cropped = image::imageops::crop_imm(&resized, crop_x, crop_y, width.min(resized.width()), height.min(resized.height())).to_image();
    canvas.blit(&cropped, x, 0);
}

fn draw_badge(canvas: &mut Canvas, card: &Card, font: Option<&Font>, x: i32, width: u32, badge_color: Color, badge_px: f32) {
    if card.badge_value.is_empty() && card.badge_label.is_empty() {
        return;
    }
    let badge_w = 298.0_f32.min(width as f32 - 32.0).max(120.0);
    let scale = badge_w / 298.0;
    let badge_h = 344.0 * scale;
    let left = x as f32 + (width as f32 - badge_w) / 2.0;
    let top = 20.0_f32;
    let vertices = [
        (148.0, 0.0), (2.0, 84.0), (0.0, 255.0),
        (151.0, 343.0), (297.0, 257.0), (297.0, 84.0),
    ]
    .into_iter()
    .map(|(vx, vy)| (left + vx * scale, top + vy * scale))
    .collect::<Vec<_>>();
    canvas.fill_polygon(&vertices, badge_color);
    let center = left + badge_w / 2.0;
    if let Some(font) = font {
        draw_centered_text(canvas, font, &card.badge_value, badge_px * scale.max(0.75), center, top + badge_h * 0.46, Color::WHITE);
        draw_centered_text(canvas, font, &card.badge_label, badge_px * 0.42 * scale.max(0.75), center, top + badge_h * 0.60, Color::WHITE);
    } else {
        draw_fallback_word(canvas, &card.badge_value, (left + 38.0 * scale) as i32, (top + badge_h * 0.35) as i32, (badge_w - 76.0 * scale) as u32, Color::WHITE);
    }
}

fn draw_card(canvas: &mut Canvas, project: &Project, card: &Card, x: i32, font: Option<&Font>) {
    let theme = &project.theme;
    let pitch = theme.card_pitch_px.round().max(2.0) as u32;
    let sep = theme.separator_px.min(pitch.saturating_sub(1));
    let card_width = pitch.saturating_sub(sep).max(1);
    let scale_y = project.export.height as f64 / 1080.0;
    let sy = |value: u32| (value as f64 * scale_y).round() as u32;
    let art_h = sy(theme.artwork_bottom_y.saturating_add(1)).min(project.export.height);
    let title_top = sy(theme.title_top_y).min(project.export.height);
    let title_bottom = sy(theme.title_bottom_y.saturating_add(1)).min(project.export.height);
    let desc_top = sy(theme.description_top_y).min(project.export.height);
    let desc_bottom = sy(theme.description_bottom_y.saturating_add(1)).min(project.export.height);

    draw_artwork(canvas, card, x, card_width, art_h);
    if title_bottom > title_top {
        canvas.fill_rect(RectI32 { x, y: title_top as i32, width: card_width, height: title_bottom - title_top }, color(card.title_panel_color));
    }
    if desc_bottom > desc_top {
        canvas.fill_rect(RectI32 { x, y: desc_top as i32, width: card_width, height: desc_bottom - desc_top }, color(card.description_panel_color));
    }
    draw_badge(canvas, card, font, x, card_width, color(theme.badge_color), theme.badge_font_px);

    let center = x as f32 + card_width as f32 / 2.0;
    if let Some(font) = font {
        let title_base = title_top as f32 + (title_bottom.saturating_sub(title_top) as f32 * 0.67);
        draw_centered_text(canvas, font, &card.title, theme.title_font_px * scale_y as f32, center, title_base, color(theme.title_text_color));
        if !card.description.is_empty() {
            let desc_base = desc_top as f32 + (desc_bottom.saturating_sub(desc_top) as f32 * 0.61);
            draw_centered_text(canvas, font, &card.description, theme.description_font_px * scale_y as f32, center, desc_base, color(theme.description_text_color));
        }
    } else {
        draw_fallback_word(canvas, &card.title, x + 20, title_top as i32 + 22, card_width.saturating_sub(40), color(theme.title_text_color));
        draw_fallback_word(canvas, &card.description, x + 20, desc_top as i32 + 28, card_width.saturating_sub(40), color(theme.description_text_color));
    }
}

/// Render one full project frame. The same Rust function is called by preview and export.
pub fn render_project_frame(project: &Project, frame: u64, font: Option<&Font>) -> Result<RgbaImage, String> {
    project.validate()?;
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
        draw_card(&mut canvas, project, card, x.round() as i32, font);
    }
    Ok(canvas.into_image())
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
    fn project_renderer_produces_exact_requested_size() {
        let mut project = Project::demo();
        project.export.width = 640;
        project.export.height = 360;
        let image = render_project_frame(&project, 0, None).unwrap();
        assert_eq!(image.dimensions(), (640, 360));
    }

    #[test]
    fn preview_png_is_valid_png() {
        let mut project = Project::demo();
        project.export.width = 320;
        project.export.height = 180;
        let png = render_project_png(&project, 12, None).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }
}
