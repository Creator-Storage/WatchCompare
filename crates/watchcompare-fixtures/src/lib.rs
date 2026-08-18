use serde::{Deserialize, Serialize};

pub const REFERENCE_TIME_BASE_DEN: u64 = 15_360;
pub const REFERENCE_TICKS_PER_FRAME: u64 = 256;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BadgeTransform {
    pub scale: f32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BellFillMetrics {
    pub inner_dark_px: u16,
    pub ring_dark_px: u16,
}

pub const fn frame_to_pts_ticks(frame: u64) -> u64 {
    frame * REFERENCE_TICKS_PER_FRAME
}

pub fn frame_to_millis(frame: u64) -> f64 {
    frame_to_pts_ticks(frame) as f64 * 1000.0 / REFERENCE_TIME_BASE_DEN as f64
}

// Second badge ("400K YEARS AGO"), local to its 477 px card.
pub const SECOND_BADGE_FIRST_VISIBLE_FRAME: u64 = 150;
pub const SECOND_BADGE_VISIBLE_CORE_BBOX: [Rect; 20] = [
    Rect { x: 0, y: 0, width: 17, height: 42 },
    Rect { x: 0, y: 0, width: 41, height: 104 },
    Rect { x: 0, y: 0, width: 65, height: 158 },
    Rect { x: 0, y: 0, width: 87, height: 202 },
    Rect { x: 0, y: 0, width: 108, height: 238 },
    Rect { x: 0, y: 0, width: 127, height: 264 },
    Rect { x: 0, y: 0, width: 141, height: 294 },
    Rect { x: 0, y: 0, width: 155, height: 304 },
    Rect { x: 0, y: 0, width: 169, height: 306 },
    Rect { x: 0, y: 0, width: 189, height: 312 },
    Rect { x: 0, y: 0, width: 201, height: 316 },
    Rect { x: 0, y: 0, width: 211, height: 318 },
    Rect { x: 0, y: 0, width: 223, height: 320 },
    Rect { x: 0, y: 0, width: 233, height: 322 },
    Rect { x: 0, y: 0, width: 241, height: 326 },
    Rect { x: 0, y: 0, width: 251, height: 328 },
    Rect { x: 0, y: 0, width: 259, height: 330 },
    Rect { x: 0, y: 0, width: 267, height: 332 },
    Rect { x: 0, y: 0, width: 275, height: 334 },
    Rect { x: 0, y: 0, width: 283, height: 336 },
];

pub const SECOND_BADGE_TRANSFORM_START_FRAME: u64 = 170;
pub const SECOND_BADGE_TRANSFORM_END_FRAME: u64 = 300;
const SECOND_BADGE_TRANSFORM_Q: &str = include_str!("second_badge_transform_q.txt");

pub fn second_badge_transform(frame: u64) -> Option<BadgeTransform> {
    if !(SECOND_BADGE_TRANSFORM_START_FRAME..=SECOND_BADGE_TRANSFORM_END_FRAME).contains(&frame) {
        return None;
    }
    let index = (frame - SECOND_BADGE_TRANSFORM_START_FRAME) as usize;
    let row = SECOND_BADGE_TRANSFORM_Q.trim().split(';').nth(index)?;
    let mut fields = row.split(',');
    let scale = fields.next()?.parse::<i16>().ok()? as f32 / 10_000.0;
    let x = fields.next()?.parse::<i16>().ok()? as f32 / 2.0;
    let y = fields.next()?.parse::<i16>().ok()? as f32 / 2.0;
    Some(BadgeTransform { scale, x, y })
}

pub const SECOND_BADGE_TEXT_FIRST_VISIBLE_FRAME: u64 = 186;
pub const SECOND_BADGE_TEXT_TRACK_END_FRAME: u64 = 265;
const SECOND_BADGE_TEXT_WHITE_AREA: &str = include_str!("second_badge_text_white_area.txt");

pub fn second_badge_text_white_area_px(frame: u64) -> Option<u16> {
    if !(SECOND_BADGE_TEXT_FIRST_VISIBLE_FRAME..=SECOND_BADGE_TEXT_TRACK_END_FRAME).contains(&frame) {
        return None;
    }
    SECOND_BADGE_TEXT_WHITE_AREA
        .trim()
        .split(',')
        .nth((frame - SECOND_BADGE_TEXT_FIRST_VISIBLE_FRAME) as usize)?
        .parse()
        .ok()
}

// CTA build and interaction timings recovered from consecutive source frames.
pub const CTA_SUBSCRIBE_FIRST_VISIBLE_FRAME: u64 = 11_930;
pub const CTA_SUBSCRIBE_GEOMETRY_SETTLED_FRAME: u64 = 11_961;
pub const CTA_LIKE_FIRST_VISIBLE_FRAME: u64 = 11_944;
pub const CTA_LIKE_GEOMETRY_SETTLED_FRAME: u64 = 12_012;
pub const CTA_BELL_FIRST_VISIBLE_FRAME: u64 = 11_946;
pub const CTA_BELL_GEOMETRY_SETTLED_FRAME: u64 = 11_978;
pub const CTA_UNDERLINE_FIRST_VISIBLE_FRAME: u64 = 11_955;
pub const CTA_UNDERLINE_GEOMETRY_SETTLED_FRAME: u64 = 11_970;
pub const CTA_DISLIKE_FIRST_VISIBLE_FRAME: u64 = 11_956;
pub const CTA_DISLIKE_GEOMETRY_SETTLED_FRAME: u64 = 12_020;
pub const CTA_CURSOR_FIRST_VISIBLE_FRAME: u64 = 12_007;
pub const CTA_LIKE_BLUE_FIRST_FRAME: u64 = 12_053;
pub const CTA_LIKE_BLUE_SETTLED_FRAME: u64 = 12_078;
pub const CTA_SUBSCRIBED_FIRST_FRAME: u64 = 12_115;
pub const CTA_BELL_FILLED_FIRST_FRAME: u64 = 12_169;

const CTA_INTERACTION_METRICS: &str = include_str!("cta_interaction_metrics.txt");

fn metric_line(prefix: &str) -> Option<&'static str> {
    CTA_INTERACTION_METRICS
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
}

pub fn cta_like_blue_level(frame: u64) -> Option<f32> {
    if !(CTA_LIKE_BLUE_FIRST_FRAME..=CTA_LIKE_BLUE_SETTLED_FRAME).contains(&frame) {
        return None;
    }
    let value = metric_line("like_blue_px=")?
        .split(',')
        .nth((frame - CTA_LIKE_BLUE_FIRST_FRAME) as usize)?
        .parse::<u16>()
        .ok()?;
    Some(value as f32 / 2209.0)
}

pub fn cta_subscribed_bbox(frame: u64) -> Option<Rect> {
    if !(CTA_SUBSCRIBED_FIRST_FRAME..=12_140).contains(&frame) {
        return None;
    }
    let row = metric_line("subscribed_gray_bbox=")?
        .split(';')
        .nth((frame - CTA_SUBSCRIBED_FIRST_FRAME) as usize)?;
    let mut fields = row.split(',');
    Some(Rect {
        x: fields.next()?.parse().ok()?,
        y: fields.next()?.parse().ok()?,
        width: fields.next()?.parse().ok()?,
        height: fields.next()?.parse().ok()?,
    })
}

pub fn cta_bell_fill_metrics(frame: u64) -> Option<BellFillMetrics> {
    if !(CTA_BELL_FILLED_FIRST_FRAME..=12_180).contains(&frame) {
        return None;
    }
    let i = (frame - CTA_BELL_FILLED_FIRST_FRAME) as usize;
    let inner_dark_px = metric_line("bell_inner_dark_px=")?.split(',').nth(i)?.parse().ok()?;
    let ring_dark_px = metric_line("bell_ring_dark_px=")?.split(',').nth(i)?.parse().ok()?;
    Some(BellFillMetrics { inner_dark_px, ring_dark_px })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_overshoots_then_settles() {
        let peak = (SECOND_BADGE_TRANSFORM_START_FRAME..=SECOND_BADGE_TRANSFORM_END_FRAME)
            .filter_map(second_badge_transform)
            .map(|v| v.scale)
            .fold(0.0_f32, f32::max);
        assert!(peak > 1.50);
        let final_sample = second_badge_transform(300).unwrap();
        assert!((final_sample.scale - 1.0).abs() < 0.01);
    }

    #[test]
    fn cta_interactions_use_exact_source_frames() {
        assert!((frame_to_millis(12_053) - 200_883.33333333334).abs() < 1e-9);
        assert!((cta_like_blue_level(12_053).unwrap() - 725.0 / 2209.0).abs() < 1e-6);
        assert_eq!(cta_like_blue_level(12_078), Some(1.0));
        assert_eq!(CTA_SUBSCRIBED_FIRST_FRAME, 12_115);
        assert_eq!(CTA_BELL_FILLED_FIRST_FRAME, 12_169);
    }
}
