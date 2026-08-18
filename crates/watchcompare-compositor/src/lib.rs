mod project_render;
pub use project_render::{render_project_frame, render_project_png};

use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};
use std::path::Path;
use watchcompare_scene::{sample_reference_scene, ReferenceSceneState};

pub const REFERENCE_WIDTH: u32 = 1920;
pub const REFERENCE_HEIGHT: u32 = 1080;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::rgba(0, 0, 0, 255);
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn pixel(self) -> Rgba<u8> {
        Rgba([self.r, self.g, self.b, self.a])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectI32 {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Canvas {
    image: RgbaImage,
}

impl Canvas {
    pub fn new(width: u32, height: u32, clear: Color) -> Self {
        Self { image: RgbaImage::from_pixel(width, height, clear.pixel()) }
    }

    pub fn reference(clear: Color) -> Self { Self::new(REFERENCE_WIDTH, REFERENCE_HEIGHT, clear) }
    pub fn width(&self) -> u32 { self.image.width() }
    pub fn height(&self) -> u32 { self.image.height() }
    pub fn as_image(&self) -> &RgbaImage { &self.image }
    pub fn into_image(self) -> RgbaImage { self.image }

    pub fn clear(&mut self, color: Color) {
        for pixel in self.image.pixels_mut() { *pixel = color.pixel(); }
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width() as i32 || y >= self.height() as i32 { return; }
        let dst = self.image.get_pixel_mut(x as u32, y as u32);
        blend_rgba(dst, color.pixel());
    }

    pub fn fill_rect(&mut self, rect: RectI32, color: Color) {
        let x0 = rect.x.max(0) as u32;
        let y0 = rect.y.max(0) as u32;
        let x1 = (rect.x.saturating_add(rect.width as i32)).max(0).min(self.width() as i32) as u32;
        let y1 = (rect.y.saturating_add(rect.height as i32)).max(0).min(self.height() as i32) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let dst = self.image.get_pixel_mut(x, y);
                blend_rgba(dst, color.pixel());
            }
        }
    }

    /// Exact integer-position blit. The source is never resampled.
    pub fn blit(&mut self, source: &RgbaImage, dst_x: i32, dst_y: i32) {
        for sy in 0..source.height() {
            let dy = dst_y + sy as i32;
            if dy < 0 || dy >= self.height() as i32 { continue; }
            for sx in 0..source.width() {
                let dx = dst_x + sx as i32;
                if dx < 0 || dx >= self.width() as i32 { continue; }
                let src = *source.get_pixel(sx, sy);
                if src[3] == 0 { continue; }
                let dst = self.image.get_pixel_mut(dx as u32, dy as u32);
                blend_rgba(dst, src);
            }
        }
    }

    pub fn fill_polygon(&mut self, vertices: &[(f32, f32)], color: Color) {
        if vertices.len() < 3 { return; }
        let min_y = vertices.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min).floor() as i32;
        let max_y = vertices.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
        for y in min_y.max(0)..=max_y.min(self.height() as i32 - 1) {
            let scan_y = y as f32 + 0.5;
            let mut intersections = Vec::with_capacity(vertices.len());
            for i in 0..vertices.len() {
                let (x0, y0) = vertices[i];
                let (x1, y1) = vertices[(i + 1) % vertices.len()];
                let crosses = (y0 <= scan_y && y1 > scan_y) || (y1 <= scan_y && y0 > scan_y);
                if crosses {
                    let t = (scan_y - y0) / (y1 - y0);
                    intersections.push(x0 + (x1 - x0) * t);
                }
            }
            intersections.sort_by(|a, b| a.total_cmp(b));
            for pair in intersections.chunks_exact(2) {
                let x0 = (pair[0] - 0.5).ceil() as i32;
                let x1 = (pair[1] - 0.5).floor() as i32;
                for x in x0.max(0)..=x1.min(self.width() as i32 - 1) { self.blend_pixel(x, y, color); }
            }
        }
    }

    pub fn diagonal_band(&mut self, clip: RectI32, axis_degrees: f32, center_normal_px: f32, half_width_px: f32, feather_px: f32, color: Color) {
        let radians = axis_degrees.to_radians();
        let normal_x = -radians.sin();
        let normal_y = radians.cos();
        let feather = feather_px.max(0.0001);
        let x0 = clip.x.max(0);
        let y0 = clip.y.max(0);
        let x1 = (clip.x + clip.width as i32).min(self.width() as i32);
        let y1 = (clip.y + clip.height as i32).min(self.height() as i32);
        for y in y0..y1 {
            for x in x0..x1 {
                let projection = (x as f32 + 0.5) * normal_x + (y as f32 + 0.5) * normal_y;
                let distance = (projection - center_normal_px).abs();
                if distance > half_width_px + feather { continue; }
                let coverage = if distance <= half_width_px { 1.0 } else { 1.0 - (distance - half_width_px) / feather };
                let alpha = (color.a as f32 * coverage).round().clamp(0.0, 255.0) as u8;
                self.blend_pixel(x, y, Color { a: alpha, ..color });
            }
        }
    }

    pub fn draw_text_baseline(&mut self, font: &Font, text: &str, px: f32, mut pen_x: f32, baseline_y: f32, color: Color) -> f32 {
        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, px);
            let glyph_x = pen_x.floor() as i32 + metrics.xmin;
            let glyph_y = baseline_y.floor() as i32 - metrics.ymin - metrics.height as i32;
            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let coverage = bitmap[gy * metrics.width + gx];
                    if coverage == 0 { continue; }
                    let alpha = ((color.a as u16 * coverage as u16 + 127) / 255) as u8;
                    self.blend_pixel(glyph_x + gx as i32, glyph_y + gy as i32, Color { a: alpha, ..color });
                }
            }
            pen_x += metrics.advance_width;
        }
        pen_x
    }
}

pub fn load_font_bytes(bytes: &[u8]) -> Result<Font, &'static str> {
    Font::from_bytes(bytes, FontSettings::default()).map_err(|_| "invalid font data")
}

pub fn load_font_file(path: impl AsRef<Path>) -> Result<Font, String> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    load_font_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn scene_for_frame(frame: u64) -> ReferenceSceneState { sample_reference_scene(frame) }

fn blend_rgba(dst: &mut Rgba<u8>, src: Rgba<u8>) {
    let sa = src[3] as u32;
    if sa == 0 { return; }
    if sa == 255 { *dst = src; return; }
    let da = dst[3] as u32;
    let inv_sa = 255 - sa;
    let out_a = sa + (da * inv_sa + 127) / 255;
    if out_a == 0 { *dst = Rgba([0, 0, 0, 0]); return; }
    let mut out = [0_u8; 4];
    for c in 0..3 {
        let src_p = src[c] as u32 * sa;
        let dst_p = dst[c] as u32 * da;
        let out_p = src_p + (dst_p * inv_sa + 127) / 255;
        out[c] = ((out_p + out_a / 2) / out_a).min(255) as u8;
    }
    out[3] = out_a.min(255) as u8;
    *dst = Rgba(out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_integer_blit_never_resamples() {
        let mut source = RgbaImage::new(2, 1);
        source.put_pixel(0, 0, Rgba([10, 20, 30, 255]));
        source.put_pixel(1, 0, Rgba([40, 50, 60, 255]));
        let mut canvas = Canvas::new(4, 2, Color::BLACK);
        canvas.blit(&source, 1, 1);
        assert_eq!(canvas.as_image().get_pixel(1, 1).0, [10, 20, 30, 255]);
        assert_eq!(canvas.as_image().get_pixel(2, 1).0, [40, 50, 60, 255]);
        assert_eq!(canvas.as_image().get_pixel(0, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn alpha_blend_is_deterministic() {
        let mut canvas = Canvas::new(1, 1, Color::BLACK);
        canvas.blend_pixel(0, 0, Color::rgba(255, 255, 255, 128));
        assert_eq!(canvas.as_image().get_pixel(0, 0).0, [128, 128, 128, 255]);
    }

    #[test]
    fn polygon_fill_stays_inside_measured_shape_bounds() {
        let mut canvas = Canvas::new(20, 20, Color::TRANSPARENT);
        canvas.fill_polygon(&[(10.0, 1.0), (18.0, 5.0), (18.0, 15.0), (10.0, 19.0), (2.0, 15.0), (2.0, 5.0)], Color::WHITE);
        assert_eq!(canvas.as_image().get_pixel(10, 10)[3], 255);
        assert_eq!(canvas.as_image().get_pixel(0, 0)[3], 0);
        assert_eq!(canvas.as_image().get_pixel(19, 19)[3], 0);
    }

    #[test]
    fn compositor_uses_unified_scene_timing() {
        let scene = scene_for_frame(3_111);
        assert!(scene.mid_video_cta.visible);
        assert_eq!(scene.mid_video_cta.phase, watchcompare_midcta::Phase::BellClick);
        let outro = scene_for_frame(12_180);
        assert!(outro.outro.fade_level < 1.0);
    }
}
