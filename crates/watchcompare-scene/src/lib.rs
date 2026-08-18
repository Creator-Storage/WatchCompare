use serde::{Deserialize, Serialize};
use watchcompare_cursor as cursor;
use watchcompare_fixtures as fx;
use watchcompare_render as render;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BadgeSceneState {
    pub visible: bool,
    pub visible_core_bbox: Option<fx::Rect>,
    pub transform: Option<fx::BadgeTransform>,
    pub text_reveal_level: f32,
    pub shine: Option<render::ShineSample>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CtaSceneState {
    pub outer_bbox: Option<render::RectU16>,
    pub subscribe_visible: bool,
    pub like_visible: bool,
    pub bell_visible: bool,
    pub underline_visible: bool,
    pub dislike_visible: bool,
    pub cursor_visible: bool,
    /// Measured topmost-white-raster cursor anchor. It is deliberately not named
    /// an OS hotspot because the fidelity target is the decoded source raster.
    pub cursor_x_px: Option<f32>,
    pub cursor_y_px: Option<f32>,
    pub like_blue_level: f32,
    pub subscribed: bool,
    pub subscribed_bbox: Option<fx::Rect>,
    pub bell_filled: bool,
    pub bell_fill_level: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OutroSceneState {
    pub wipe_bottom_y: Option<u16>,
    pub group: Option<render::OutroGroupSample>,
    pub fade_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceSceneState {
    pub frame: u64,
    pub pts_ticks: u64,
    pub time_millis: f64,
    pub stage: render::TimelineStage,
    pub card_train_x_px: f64,
    pub card_phase_px: f64,
    pub first_card_reveal_width_px: u16,
    pub credits_left_x_px: Option<u16>,
    pub second_badge: BadgeSceneState,
    pub cta: CtaSceneState,
    pub outro: OutroSceneState,
}

fn second_badge_visible_core_bbox(frame: u64) -> Option<fx::Rect> {
    if frame < fx::SECOND_BADGE_FIRST_VISIBLE_FRAME {
        return None;
    }
    if frame < fx::SECOND_BADGE_TRANSFORM_START_FRAME {
        return fx::SECOND_BADGE_VISIBLE_CORE_BBOX
            .get((frame - fx::SECOND_BADGE_FIRST_VISIBLE_FRAME) as usize)
            .copied();
    }
    None
}

fn second_badge_transform(frame: u64) -> Option<fx::BadgeTransform> {
    if frame < fx::SECOND_BADGE_TRANSFORM_START_FRAME {
        return None;
    }
    fx::second_badge_transform(frame.min(fx::SECOND_BADGE_TRANSFORM_END_FRAME))
}

fn second_badge_text_reveal_level(frame: u64) -> f32 {
    const FINAL_WHITE_AREA: f32 = 12_707.0;
    if frame < fx::SECOND_BADGE_TEXT_FIRST_VISIBLE_FRAME {
        return 0.0;
    }
    if frame > fx::SECOND_BADGE_TEXT_TRACK_END_FRAME {
        return 1.0;
    }
    fx::second_badge_text_white_area_px(frame)
        .map(|v| (v as f32 / FINAL_WHITE_AREA).clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

fn cta_like_blue_level(frame: u64) -> f32 {
    if frame < fx::CTA_LIKE_BLUE_FIRST_FRAME {
        0.0
    } else if frame > fx::CTA_LIKE_BLUE_SETTLED_FRAME {
        1.0
    } else {
        fx::cta_like_blue_level(frame).unwrap_or(0.0)
    }
}

fn cta_subscribed_bbox(frame: u64) -> Option<fx::Rect> {
    if frame < fx::CTA_SUBSCRIBED_FIRST_FRAME {
        None
    } else if frame <= 12_140 {
        fx::cta_subscribed_bbox(frame)
    } else {
        fx::cta_subscribed_bbox(12_140)
    }
}

fn cta_bell_fill_level(frame: u64) -> f32 {
    const FINAL_DARK_PX: f32 = 866.0;
    if frame < fx::CTA_BELL_FILLED_FIRST_FRAME {
        0.0
    } else if frame > 12_180 {
        1.0
    } else {
        fx::cta_bell_fill_metrics(frame)
            .map(|v| (v.inner_dark_px as f32 / FINAL_DARK_PX).clamp(0.0, 1.0))
            .unwrap_or(0.0)
    }
}

pub fn sample_reference_scene(frame: u64) -> ReferenceSceneState {
    let base = render::sample_reference_frame(frame);
    let frame = base.frame;
    let cursor_tip = cursor::cursor_white_tip(frame);

    let second_badge = BadgeSceneState {
        visible: frame >= fx::SECOND_BADGE_FIRST_VISIBLE_FRAME,
        visible_core_bbox: second_badge_visible_core_bbox(frame),
        transform: second_badge_transform(frame),
        text_reveal_level: second_badge_text_reveal_level(frame),
        shine: render::second_badge_shine_sample(frame),
    };

    let cta = CtaSceneState {
        outer_bbox: render::outro_cta_bbox(frame),
        subscribe_visible: frame >= fx::CTA_SUBSCRIBE_FIRST_VISIBLE_FRAME,
        like_visible: frame >= fx::CTA_LIKE_FIRST_VISIBLE_FRAME,
        bell_visible: frame >= fx::CTA_BELL_FIRST_VISIBLE_FRAME,
        underline_visible: frame >= fx::CTA_UNDERLINE_FIRST_VISIBLE_FRAME,
        dislike_visible: frame >= fx::CTA_DISLIKE_FIRST_VISIBLE_FRAME,
        cursor_visible: cursor_tip.is_some(),
        cursor_x_px: cursor_tip.map(|tip| tip.x as f32),
        cursor_y_px: cursor_tip.map(|tip| tip.y as f32),
        like_blue_level: cta_like_blue_level(frame),
        subscribed: frame >= fx::CTA_SUBSCRIBED_FIRST_FRAME,
        subscribed_bbox: cta_subscribed_bbox(frame),
        bell_filled: frame >= fx::CTA_BELL_FILLED_FIRST_FRAME,
        bell_fill_level: cta_bell_fill_level(frame),
    };

    let outro = OutroSceneState {
        wipe_bottom_y: render::outro_wipe_bottom_y(frame),
        group: render::outro_group_sample(frame),
        fade_level: render::outro_fade_level(frame),
    };

    ReferenceSceneState {
        frame,
        pts_ticks: base.pts_ticks,
        time_millis: base.time_millis,
        stage: base.stage,
        card_train_x_px: base.card_train_x_px,
        card_phase_px: base.card_phase_px,
        first_card_reveal_width_px: render::first_card_reveal_width_px(frame),
        credits_left_x_px: render::credits_left_x_px(frame),
        second_badge,
        cta,
        outro,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_badge_uses_source_frame_state() {
        let first = sample_reference_scene(150);
        assert!(first.second_badge.visible);
        assert_eq!(first.second_badge.visible_core_bbox.unwrap().width, 17);
        assert!(first.second_badge.transform.is_none());
        let overshoot = sample_reference_scene(174);
        assert!(overshoot.second_badge.transform.unwrap().scale > 1.52);
        let settled = sample_reference_scene(300);
        assert!((settled.second_badge.transform.unwrap().scale - 1.0).abs() < 0.01);
    }

    #[test]
    fn cta_click_states_are_timestamped_not_generic_tweens() {
        assert_eq!(sample_reference_scene(12_052).cta.like_blue_level, 0.0);
        assert!(sample_reference_scene(12_053).cta.like_blue_level > 0.3);
        assert_eq!(sample_reference_scene(12_078).cta.like_blue_level, 1.0);
        assert!(!sample_reference_scene(12_114).cta.subscribed);
        assert!(sample_reference_scene(12_115).cta.subscribed);
        assert!(!sample_reference_scene(12_168).cta.bell_filled);
        assert!(sample_reference_scene(12_169).cta.bell_filled);
    }

    #[test]
    fn cursor_uses_consecutive_source_raster_track() {
        let first = sample_reference_scene(cursor::CURSOR_FIRST_VISIBLE_FRAME);
        assert!(first.cta.cursor_visible);
        assert_eq!(first.cta.cursor_x_px, Some(530.0));
        assert_eq!(first.cta.cursor_y_px, Some(221.0));

        let jump = sample_reference_scene(12_078);
        assert_eq!(jump.cta.cursor_x_px, Some(678.0));
        assert_eq!(jump.cta.cursor_y_px, Some(123.0));

        let exit = sample_reference_scene(cursor::CURSOR_TRACK_END_FRAME);
        assert_eq!(exit.cta.cursor_x_px, Some(1003.0));
        assert_eq!(exit.cta.cursor_y_px, Some(282.0));
        assert!(!sample_reference_scene(cursor::CURSOR_GONE_FRAME).cta.cursor_visible);
    }
}
