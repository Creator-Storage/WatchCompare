use serde::{Deserialize, Serialize};

pub const REFERENCE_WIDTH: u32 = 1920;
pub const REFERENCE_HEIGHT: u32 = 1080;
pub const REFERENCE_FPS: f64 = 60.0;
pub const REFERENCE_FRAME_COUNT: u64 = 12_267;
pub const REFERENCE_DURATION_SECONDS: f64 = 204.45;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ReferenceGeometry {
    pub card_pitch_px: f64,
    pub artwork_bottom_y: u32,
    pub title_top_y: u32,
    pub title_bottom_y: u32,
    pub description_top_y: u32,
    pub description_bottom_y: u32,
    pub bottom_border_top_y: u32,
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
            status: "measurement_pass_1",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameState {
    pub frame: u64,
    pub time_seconds: f64,
    pub stage: TimelineStage,
    pub card_train_x_px: f64,
    pub card_phase_px: f64,
}

pub fn frame_to_seconds(frame: u64) -> f64 {
    frame as f64 / REFERENCE_FPS
}

pub fn seconds_to_frame(seconds: f64) -> u64 {
    (seconds.max(0.0) * REFERENCE_FPS).round() as u64
}

pub fn sample_reference_frame(frame: u64) -> FrameState {
    let profile = ReferenceProfile::default();
    let frame = frame.min(REFERENCE_FRAME_COUNT.saturating_sub(1));
    let time_seconds = frame_to_seconds(frame);

    let stage = if time_seconds < 12.0 {
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
    fn steady_motion_moves_left() {
        let a = sample_reference_frame(1200);
        let b = sample_reference_frame(1230);
        let delta = b.card_train_x_px - a.card_train_x_px;
        assert!((delta + 66.736_563_946_893_21).abs() < 1e-9);
    }
}
