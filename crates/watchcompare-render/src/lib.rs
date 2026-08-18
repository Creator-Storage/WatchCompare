use serde::{Deserialize, Serialize};

pub const REFERENCE_WIDTH: u32 = 1920;
pub const REFERENCE_HEIGHT: u32 = 1080;
pub const REFERENCE_FPS: f64 = 60.0;
pub const REFERENCE_FRAME_COUNT: u64 = 12_267;
pub const REFERENCE_DURATION_SECONDS: f64 = 204.45;

/// The MP4 stream uses a 1/15360 second video time base and every source frame
/// advances exactly 256 ticks. Keeping this integer clock avoids accumulating
/// floating-point timing drift while reproducing the reference.
pub const REFERENCE_TIME_BASE_DEN: u64 = 15_360;
pub const REFERENCE_TICKS_PER_FRAME: u64 = 256;
pub const REFERENCE_FRAME_DURATION_MILLIS: f64 = 50.0 / 3.0;

/// Geometry measured directly from steady-state frames in the supplied reference.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ReferenceGeometry {
    /// Repeating distance between neighboring card separators.
    pub card_pitch_px: f64,
    /// Artwork ends on row 871. The title panel begins at row 872.
    pub artwork_bottom_y: u32,
    pub title_top_y: u32,
    pub title_bottom_y: u32,
    pub description_top_y: u32,
    pub description_bottom_y: u32,
    pub bottom_border_top_y: u32,
    /// Typical visible vertical separator width. It varies by antialias/compression edge pixels.
    pub separator_nominal_px: f64,
}

impl Default for ReferenceGeometry {
    fn default() -> Self {
        Self {
            card_pitch_px: 477.0,
            artwork_bottom_y: 871,
            title_top_y: 872,
            title_bottom_y: 964,
            description_top_y: 965,
            description_bottom_y: 1074,
            bottom_border_top_y: 1075,
            separator_nominal_px: 6.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ReferenceMotion {
    /// Mean horizontal displacement measured with phase correlation across
    /// three 0.5 s steady-state windows. A full every-frame pass is now used
    /// to recover the non-linear intro acceleration independently.
    pub steady_scroll_px_per_second: f64,
    pub steady_scroll_px_per_frame: f64,
}

impl Default for ReferenceMotion {
    fn default() -> Self {
        let px_per_second = 133.473_127_893_786_43;
        Self {
            steady_scroll_px_per_second: px_per_second,
            steady_scroll_px_per_frame: px_per_second / REFERENCE_FPS,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimelineStage {
    Intro,
    Cruise,
    Outro,
    Fade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceProfile {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub frame_count: u64,
    pub duration_seconds: f64,
    pub geometry: ReferenceGeometry,
    pub motion: ReferenceMotion,
    pub status: &'static str,
}

impl Default for ReferenceProfile {
    fn default() -> Self {
        Self {
            width: REFERENCE_WIDTH,
            height: REFERENCE_HEIGHT,
            fps: REFERENCE_FPS,
            frame_count: REFERENCE_FRAME_COUNT,
            duration_seconds: REFERENCE_DURATION_SECONDS,
            geometry: ReferenceGeometry::default(),
            motion: ReferenceMotion::default(),
            status: "measurement_pass_2_exact_pts",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameState {
    pub frame: u64,
    /// Exact source clock position before conversion to floating-point units.
    pub pts_ticks: u64,
    pub time_millis: f64,
    pub time_seconds: f64,
    pub stage: TimelineStage,
    /// Continuous translation of the card train during the steady state.
    /// Intro and outro receive dedicated measured curves during reference lock.
    pub card_train_x_px: f64,
    /// Card phase within one measured 477 px pitch.
    pub card_phase_px: f64,
}

pub const fn frame_to_pts_ticks(frame: u64) -> u64 {
    frame * REFERENCE_TICKS_PER_FRAME
}

pub fn pts_ticks_to_seconds(ticks: u64) -> f64 {
    ticks as f64 / REFERENCE_TIME_BASE_DEN as f64
}

pub fn pts_ticks_to_millis(ticks: u64) -> f64 {
    ticks as f64 * 1000.0 / REFERENCE_TIME_BASE_DEN as f64
}

pub fn frame_to_seconds(frame: u64) -> f64 {
    pts_ticks_to_seconds(frame_to_pts_ticks(frame))
}

pub fn frame_to_millis(frame: u64) -> f64 {
    pts_ticks_to_millis(frame_to_pts_ticks(frame))
}

pub fn seconds_to_frame(seconds: f64) -> u64 {
    (seconds.max(0.0) * REFERENCE_FPS).round() as u64
}

/// Samples the currently verified steady-state motion model on the exact source
/// frame clock. The intro/card reveal and outro curves are intentionally isolated
/// until their per-frame measurements are promoted to renderer fixtures.
pub fn sample_reference_frame(frame: u64) -> FrameState {
    let profile = ReferenceProfile::default();
    let frame = frame.min(REFERENCE_FRAME_COUNT.saturating_sub(1));
    let pts_ticks = frame_to_pts_ticks(frame);
    let time_seconds = pts_ticks_to_seconds(pts_ticks);
    let time_millis = pts_ticks_to_millis(pts_ticks);

    // The every-frame separator pass shows the intro pan begins at frame 524
    // (8733.333 ms), overshoots cruise speed, and has settled around the normal
    // scroll rate by about frame 630 (10500 ms). Until the measured pan curve is
    // installed, keep all of it in Intro instead of pretending it is linear.
    let stage = if frame < 630 {
        TimelineStage::Intro
    } else if time_seconds < 194.0 {
        TimelineStage::Cruise
    } else if time_seconds < 202.8 {
        TimelineStage::Outro
    } else {
        TimelineStage::Fade
    };

    let x = -profile.motion.steady_scroll_px_per_frame * frame as f64;
    let pitch = profile.geometry.card_pitch_px;
    let card_phase_px = ((x % pitch) + pitch) % pitch;

    FrameState {
        frame,
        pts_ticks,
        time_millis,
        time_seconds,
        stage,
        card_train_x_px: x,
        card_phase_px,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_geometry_is_self_consistent() {
        let g = ReferenceGeometry::default();
        assert_eq!(g.title_top_y, g.artwork_bottom_y + 1);
        assert_eq!(g.description_top_y, g.title_bottom_y + 1);
        assert_eq!(g.bottom_border_top_y, g.description_bottom_y + 1);
        assert!(g.card_pitch_px > 470.0 && g.card_pitch_px < 485.0);
    }

    #[test]
    fn source_clock_is_exactly_sixty_fps() {
        assert_eq!(frame_to_pts_ticks(0), 0);
        assert_eq!(frame_to_pts_ticks(1), 256);
        assert_eq!(frame_to_pts_ticks(12_266), 3_140_096);
        assert!((frame_to_millis(1) - 50.0 / 3.0).abs() < 1e-12);
        assert!((frame_to_millis(60) - 1000.0).abs() < 1e-12);
    }

    #[test]
    fn steady_motion_moves_left() {
        let a = sample_reference_frame(1200);
        let b = sample_reference_frame(1230);
        let delta = b.card_train_x_px - a.card_train_x_px;
        assert!((delta + 66.736_563_946_893_21).abs() < 1e-9);
    }

    #[test]
    fn measured_intro_pan_stays_out_of_cruise_until_frame_630() {
        assert_eq!(sample_reference_frame(629).stage, TimelineStage::Intro);
        assert_eq!(sample_reference_frame(630).stage, TimelineStage::Cruise);
    }
}
