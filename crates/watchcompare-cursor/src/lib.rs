use serde::{Deserialize, Serialize};

pub const CURSOR_FIRST_VISIBLE_FRAME: u64 = 12_007;
pub const CURSOR_TRACK_END_FRAME: u64 = 12_180;
pub const CURSOR_TRACK_FRAME_COUNT: usize =
    (CURSOR_TRACK_END_FRAME - CURSOR_FIRST_VISIBLE_FRAME + 1) as usize;

const CURSOR_WHITE_TIP_XY: &str = include_str!("cursor_white_tip_xy.txt");

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorWhiteTip {
    pub x: u16,
    pub y: u16,
}

/// Returns the exact topmost-white-pixel anchor measured from the decoded
/// reference cursor raster on this source frame.
///
/// This deliberately does not claim to be the operating-system cursor hotspot.
/// Matching the source raster is the useful invariant for the compositor.
pub fn cursor_white_tip(frame: u64) -> Option<CursorWhiteTip> {
    if !(CURSOR_FIRST_VISIBLE_FRAME..=CURSOR_TRACK_END_FRAME).contains(&frame) {
        return None;
    }
    let row = CURSOR_WHITE_TIP_XY
        .trim()
        .split(';')
        .nth((frame - CURSOR_FIRST_VISIBLE_FRAME) as usize)?;
    let (x, y) = row.split_once(',')?;
    Some(CursorWhiteTip {
        x: x.parse().ok()?,
        y: y.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_consecutive_source_track_is_present() {
        assert_eq!(CURSOR_WHITE_TIP_XY.trim().split(';').count(), CURSOR_TRACK_FRAME_COUNT);
        assert_eq!(cursor_white_tip(12_006), None);
        assert_eq!(cursor_white_tip(12_007), Some(CursorWhiteTip { x: 530, y: 221 }));
        assert_eq!(cursor_white_tip(12_053), Some(CursorWhiteTip { x: 543, y: 103 }));
        assert_eq!(cursor_white_tip(12_078), Some(CursorWhiteTip { x: 678, y: 123 }));
        assert_eq!(cursor_white_tip(12_115), Some(CursorWhiteTip { x: 854, y: 108 }));
        assert_eq!(cursor_white_tip(12_169), Some(CursorWhiteTip { x: 963, y: 110 }));
        assert_eq!(cursor_white_tip(12_180), Some(CursorWhiteTip { x: 962, y: 110 }));
        assert_eq!(cursor_white_tip(12_181), None);
    }
}
