use serde::{Deserialize, Serialize};

pub const MID_CTA_FIRST_VISIBLE_FRAME: u64 = 2_931;
pub const MID_CTA_RED_LAST_FRAME: u64 = 3_161;
pub const MID_CTA_LAST_VISIBLE_FRAME: u64 = 3_185;
pub const MID_CTA_GONE_FRAME: u64 = 3_186;

pub const MID_CTA_PLAY_CIRCLE_FULL_FRAME: u64 = 2_941;
pub const MID_CTA_PILL_FULL_FRAME: u64 = 2_976;
pub const MID_CTA_CURSOR_FIRST_VISIBLE_FRAME: u64 = 3_028;
pub const MID_CTA_PLAY_CLICK_BURST_START_FRAME: u64 = 3_061;
pub const MID_CTA_SWEEP_BLUR_START_FRAME: u64 = 3_066;
pub const MID_CTA_PILL_COLLAPSE_START_FRAME: u64 = 3_073;
pub const MID_CTA_SUBSCRIBED_TEXT_FIRST_VISIBLE_FRAME: u64 = 3_078;
pub const MID_CTA_SUBSCRIBED_TEXT_SETTLED_FRAME: u64 = 3_083;
pub const MID_CTA_RIGHT_PLAY_SETTLED_FRAME: u64 = 3_086;
pub const MID_CTA_BELL_CLICK_GLOW_START_FRAME: u64 = 3_111;
pub const MID_CTA_SECOND_RED_CONTROL_START_FRAME: u64 = 3_116;
pub const MID_CTA_TWO_RED_CONTROLS_SETTLED_FRAME: u64 = 3_118;
pub const MID_CTA_CURSOR_LAST_VISIBLE_FRAME: u64 = 3_120;
pub const MID_CTA_CURSOR_GONE_FRAME: u64 = 3_121;

const RED_BBOX_XYWH: &str = include_str!("red_bbox_xywh.txt");

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Hidden,
    Entering,
    SubscribeIdle,
    SubscribeClick,
    SubscribedMorph,
    SubscribedIdle,
    BellClick,
    BellActive,
    GlowExit,
}

/// Exact bounding box of the tracked red CTA raster at the source frame.
/// Frame 3161 is already in the pale/glow transition but still belongs to the
/// measured red track. Frames 3162..3185 are the final white/glow-only exit.
pub fn red_bbox(frame: u64) -> Option<Rect> {
    if !(MID_CTA_FIRST_VISIBLE_FRAME..=MID_CTA_RED_LAST_FRAME).contains(&frame) {
        return None;
    }
    let row = RED_BBOX_XYWH
        .trim()
        .split(';')
        .nth((frame - MID_CTA_FIRST_VISIBLE_FRAME) as usize)?;
    let mut fields = row.split(',');
    Some(Rect {
        x: fields.next()?.parse().ok()?,
        y: fields.next()?.parse().ok()?,
        width: fields.next()?.parse().ok()?,
        height: fields.next()?.parse().ok()?,
    })
}

pub const fn visible(frame: u64) -> bool {
    frame >= MID_CTA_FIRST_VISIBLE_FRAME && frame <= MID_CTA_LAST_VISIBLE_FRAME
}

pub const fn cursor_visible(frame: u64) -> bool {
    frame >= MID_CTA_CURSOR_FIRST_VISIBLE_FRAME && frame <= MID_CTA_CURSOR_LAST_VISIBLE_FRAME
}

/// Source-frame phase boundaries recovered from consecutive decoded frames.
/// This does not invent easing between frames; it names the already-observed
/// raster regimes so the compositor can use the correct exact fixture set.
pub const fn phase(frame: u64) -> Phase {
    if frame < MID_CTA_FIRST_VISIBLE_FRAME || frame >= MID_CTA_GONE_FRAME {
        Phase::Hidden
    } else if frame < MID_CTA_PILL_FULL_FRAME {
        Phase::Entering
    } else if frame < MID_CTA_PLAY_CLICK_BURST_START_FRAME {
        Phase::SubscribeIdle
    } else if frame < MID_CTA_SUBSCRIBED_TEXT_FIRST_VISIBLE_FRAME {
        Phase::SubscribeClick
    } else if frame < MID_CTA_RIGHT_PLAY_SETTLED_FRAME {
        Phase::SubscribedMorph
    } else if frame < MID_CTA_BELL_CLICK_GLOW_START_FRAME {
        Phase::SubscribedIdle
    } else if frame < MID_CTA_TWO_RED_CONTROLS_SETTLED_FRAME {
        Phase::BellClick
    } else if frame <= MID_CTA_RED_LAST_FRAME {
        Phase::BellActive
    } else {
        Phase::GlowExit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_red_track_has_every_source_frame() {
        assert_eq!(
            RED_BBOX_XYWH.trim().split(';').count(),
            (MID_CTA_RED_LAST_FRAME - MID_CTA_FIRST_VISIBLE_FRAME + 1) as usize
        );
        assert_eq!(red_bbox(2_930), None);
        assert_eq!(red_bbox(2_931), Some(Rect { x: 106, y: 966, width: 12, height: 10 }));
        assert_eq!(red_bbox(2_941), Some(Rect { x: 58, y: 916, width: 110, height: 110 }));
        assert_eq!(red_bbox(2_976), Some(Rect { x: 58, y: 916, width: 462, height: 110 }));
        assert_eq!(red_bbox(3_086), Some(Rect { x: 414, y: 920, width: 101, height: 102 }));
        assert_eq!(red_bbox(3_118), Some(Rect { x: 418, y: 925, width: 203, height: 91 }));
        assert_eq!(red_bbox(3_161), Some(Rect { x: 418, y: 925, width: 203, height: 92 }));
        assert_eq!(red_bbox(3_162), None);
        assert!(visible(3_185));
        assert_eq!(red_bbox(3_185), None);
        assert!(!visible(MID_CTA_GONE_FRAME));
    }

    #[test]
    fn internal_misc_boundaries_are_source_frame_exact() {
        assert!(!cursor_visible(3_027));
        assert!(cursor_visible(3_028));
        assert!(cursor_visible(3_120));
        assert!(!cursor_visible(3_121));

        assert_eq!(phase(2_930), Phase::Hidden);
        assert_eq!(phase(2_931), Phase::Entering);
        assert_eq!(phase(2_976), Phase::SubscribeIdle);
        assert_eq!(phase(3_061), Phase::SubscribeClick);
        assert_eq!(phase(3_078), Phase::SubscribedMorph);
        assert_eq!(phase(3_086), Phase::SubscribedIdle);
        assert_eq!(phase(3_111), Phase::BellClick);
        assert_eq!(phase(3_118), Phase::BellActive);
        assert_eq!(phase(3_162), Phase::GlowExit);
        assert_eq!(phase(3_186), Phase::Hidden);
    }
}
