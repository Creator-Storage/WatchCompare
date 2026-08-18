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

/// Canonical red badge raster measured from a stable, shine-free first-badge frame.
/// Coordinates are relative to the component's top-left raster extent.
pub const BADGE_CANONICAL_RED_WIDTH: u32 = 298;
pub const BADGE_CANONICAL_RED_HEIGHT: u32 = 344;
pub const BADGE_CANONICAL_VERTICES: [(i32, i32); 6] = [
    (148, 0),
    (2, 84),
    (0, 255),
    (151, 343),
    (297, 257),
    (297, 84),
];

/// Exact source-frame events measured from consecutive decoded frames.
pub const FIRST_BADGE_FIRST_VISIBLE_FRAME: u64 = 34; // 566.666... ms
pub const SECOND_CARD_FIRST_VISIBLE_FRAME: u64 = 125; // 2083.333... ms
pub const THIRD_CARD_FIRST_VISIBLE_FRAME: u64 = 244; // 4066.666... ms
pub const CREDITS_RETRACT_START_FRAME: u64 = 396; // 6600 ms
pub const CREDITS_GONE_FRAME: u64 = 429; // 7150 ms
pub const INTRO_PAN_FIRST_MOVING_FRAME: u64 = 524; // 8733.333... ms
pub const INTRO_PAN_SETTLED_FRAME: u64 = 630; // about 10500 ms

/// Second-badge diagonal-shine window. The broad soft band has a faint lead-in
/// before the strong interval and is visually gone by frame 252.
pub const SECOND_BADGE_SHINE_FAINT_START_FRAME: u64 = 232;
pub const SECOND_BADGE_SHINE_STRONG_START_FRAME: u64 = 234;
pub const SECOND_BADGE_SHINE_STRONG_END_FRAME: u64 = 249;
pub const SECOND_BADGE_SHINE_GONE_FRAME: u64 = 252;

/// The end-screen remains at full measured brightness through frame 12179.
/// Its first fading source image is frame 12180 (203000 ms), and the tracked
/// end-screen red region is fully black from frame 12258 onward.
pub const OUTRO_FADE_FIRST_FRAME: u64 = 12_180;
pub const OUTRO_BLACK_FIRST_FRAME: u64 = 12_258;

/// Exact measured title-panel reveal width for card 1, frames 0..=89.
/// Values are source pixels at row y=924. The card remains at 480 px after
/// this fixture interval.
pub const FIRST_CARD_REVEAL_WIDTH_PX: [u16; 90] = [
    0, 0, 0, 0, 0, 9, 12, 17, 22, 29, 37, 47, 60, 75, 93, 114, 137, 160,
    183, 204, 224, 242, 258, 273, 286, 298, 310, 320, 330, 339, 348, 355, 363,
    370, 376, 382, 388, 394, 399, 404, 408, 413, 417, 421, 424, 428, 431, 434,
    437, 440, 443, 446, 448, 450, 453, 455, 457, 459, 460, 462, 464, 465, 467,
    468, 469, 470, 471, 472, 473, 474, 475, 476, 476, 477, 477, 478, 478, 479,
    479, 479, 479, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480,
];

/// Card-train x translation measured from the separator that is stationary at
/// x=960 through frame 523. Values are in half-source-pixels for frames
/// 523..=630, preserving the decoded raster's 0.5 px edge positions.
pub const INTRO_PAN_TRACK_START_FRAME: u64 = 523;
pub const INTRO_PAN_HALF_PX: [i16; 108] = [
    0, -1, -1, -3, -3, -5, -7, -9, -11, -13, -16, -18, -22, -25, -27, -32,
    -35, -40, -43, -45, -51, -55, -61, -64, -67, -75, -78, -85, -89, -93,
    -100, -104, -112, -116, -121, -129, -133, -141, -146, -151, -159, -164,
    -173, -178, -183, -192, -197, -207, -211, -217, -227, -231, -242, -247,
    -252, -263, -268, -278, -283, -289, -299, -305, -316, -321, -327, -338,
    -343, -355, -360, -366, -377, -383, -394, -400, -405, -417, -423, -434,
    -439, -445, -457, -463, -474, -479, -485, -497, -502, -513, -519, -524,
    -535, -541, -551, -556, -561, -571, -575, -584, -587, -590, -596, -599,
    -605, -609, -612, -618, -621, -627,
];

/// Credits overlay left edge for frames 396..=428. Frame 429 is the first
/// frame with no detected credits panel at the tracked row.
pub const CREDITS_EDGE_TRACK_START_FRAME: u64 = 396;
pub const CREDITS_LEFT_X_PX: [u16; 33] = [
    1703, 1717, 1730, 1741, 1752, 1762, 1771, 1779, 1788, 1795, 1800, 1810,
    1816, 1822, 1826, 1825, 1838, 1843, 1848, 1852, 1856, 1860, 1864, 1868,
    1871, 1874, 1877, 1880, 1883, 1886, 1888, 1891, 1893,
];

/// Measured diagonal-shine band geometry for the second badge, frames 232..=251.
/// `center` is the fitted band-normal coordinate; `width80` is the central 80%
/// bright-band span in source pixels. Timing is verified; geometry remains a
/// source-derived fixture until compositor image diffs lock the exact shader.
pub const SECOND_BADGE_SHINE_TRACK_START_FRAME: u64 = 232;
pub const SECOND_BADGE_SHINE_NORMAL_CENTER_PX: [f32; 20] = [
    127.516f32, 134.185f32, 142.862f32, 151.458f32, 162.104f32, 170.791f32, 182.513f32, 201.631f32, 217.594f32, 232.1f32, 255.732f32, 275.272f32, 297.427f32, 323.617f32, 343.087f32, 367.092f32, 388.148f32, 409.323f32, 422.615f32, 441.565f32
];
pub const SECOND_BADGE_SHINE_WIDTH80_PX: [f32; 20] = [
    12.2988f32, 15.6695f32, 22.9638f32, 29.5085f32, 37.0457f32, 37.4012f32, 39.9066f32, 41.3882f32, 39.3963f32, 38.8319f32, 39.6867f32, 34.3059f32, 32.1609f32, 32.0696f32, 26.8857f32, 27.4303f32, 20.9928f32, 16.1293f32, 37.0101f32, 16.6275f32
];

/// Measured normalized end-screen red-region intensity, frames 12179..=12258,
/// quantized to 0..=10000. This is a source brightness fixture, not an assumed
/// linear alpha curve. Frames before it are 1.0; frames after it are 0.0.
pub const OUTRO_FADE_TRACK_START_FRAME: u64 = 12_179;
pub const OUTRO_FADE_LEVEL_10000: [u16; 80] = [
    10000, 9904, 9766, 9726, 9726, 9726, 9483, 9483, 9441, 9303, 9303, 9204, 9204, 9204, 8876, 8876, 8738, 8455, 8411, 8416, 7986, 7986, 7986, 7932, 7570, 7570, 7430, 7239, 7239, 7101, 6918, 6816, 6580, 6580, 6351, 6351, 6157, 6157, 6017, 5833, 5643, 5599, 5360, 5360, 5360, 5223, 4753, 4753, 4709, 4615, 4475, 4279, 4279, 4139, 3906, 3906, 3763, 3670, 3527, 3389, 3296, 3204, 3066, 3013, 2777, 2777, 2639, 2499, 2116, 2261, 2030, 1976, 1840, 1700, 1556, 1411, 1273, 1035, 705, 0
];


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ShineSample {
    pub normal_center_px: f32,
    pub width80_px: f32,
}

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
            status: "measurement_pass_3_exact_intro_badge_outro_fade",
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


/// Exact source-frame first-card reveal width measured at the title-panel row.
pub fn first_card_reveal_width_px(frame: u64) -> u16 {
    FIRST_CARD_REVEAL_WIDTH_PX
        .get(frame as usize)
        .copied()
        .unwrap_or(480)
}

/// Exact measured card-train x position through the intro pan, then a continuous
/// cruise extension anchored to the last measured intro frame.
pub fn measured_card_train_x_px(frame: u64) -> f64 {
    if frame <= INTRO_PAN_TRACK_START_FRAME {
        return 0.0;
    }

    if frame <= INTRO_PAN_SETTLED_FRAME {
        let index = (frame - INTRO_PAN_TRACK_START_FRAME) as usize;
        return INTRO_PAN_HALF_PX[index] as f64 / 2.0;
    }

    let anchor = INTRO_PAN_HALF_PX[INTRO_PAN_HALF_PX.len() - 1] as f64 / 2.0;
    let elapsed_frames = frame - INTRO_PAN_SETTLED_FRAME;
    anchor - ReferenceMotion::default().steady_scroll_px_per_frame * elapsed_frames as f64
}

/// Returns the measured credits-overlay left edge. `None` means the panel is
/// fully gone at the tracked row. Before retraction it is stationary at x=1431.
pub fn credits_left_x_px(frame: u64) -> Option<u16> {
    if frame < CREDITS_RETRACT_START_FRAME {
        return Some(1431);
    }
    if frame >= CREDITS_GONE_FRAME {
        return None;
    }
    let index = (frame - CREDITS_EDGE_TRACK_START_FRAME) as usize;
    CREDITS_LEFT_X_PX.get(index).copied()
}

/// Returns the consecutive-frame fitted shine-band geometry for the second badge.
pub fn second_badge_shine_sample(frame: u64) -> Option<ShineSample> {
    if !(SECOND_BADGE_SHINE_TRACK_START_FRAME..SECOND_BADGE_SHINE_GONE_FRAME).contains(&frame) {
        return None;
    }
    let index = (frame - SECOND_BADGE_SHINE_TRACK_START_FRAME) as usize;
    Some(ShineSample {
        normal_center_px: SECOND_BADGE_SHINE_NORMAL_CENTER_PX[index],
        width80_px: SECOND_BADGE_SHINE_WIDTH80_PX[index],
    })
}

/// Returns the measured end-screen fade brightness fixture on the exact source frame.
/// This intentionally preserves plateaus and compression/raster steps from the source.
pub fn outro_fade_level(frame: u64) -> f64 {
    if frame < OUTRO_FADE_TRACK_START_FRAME {
        return 1.0;
    }
    let index = (frame - OUTRO_FADE_TRACK_START_FRAME) as usize;
    OUTRO_FADE_LEVEL_10000
        .get(index)
        .map(|value| *value as f64 / 10_000.0)
        .unwrap_or(0.0)
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
    // scroll rate by about frame 630 (10500 ms). The measured half-pixel lookup
    // above is used directly through that settle frame rather than a generic easing.
    let stage = if frame < INTRO_PAN_SETTLED_FRAME {
        TimelineStage::Intro
    } else if time_seconds < 194.0 {
        TimelineStage::Cruise
    } else if frame < OUTRO_FADE_FIRST_FRAME {
        TimelineStage::Outro
    } else {
        TimelineStage::Fade
    };

    let x = measured_card_train_x_px(frame);
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

    #[test]
    fn measured_intro_events_land_on_exact_source_clock() {
        assert!((frame_to_millis(FIRST_BADGE_FIRST_VISIBLE_FRAME) - 566.666_666_666_666_6).abs() < 1e-9);
        assert!((frame_to_millis(SECOND_CARD_FIRST_VISIBLE_FRAME) - 2083.333_333_333_333_5).abs() < 1e-9);
        assert!((frame_to_millis(THIRD_CARD_FIRST_VISIBLE_FRAME) - 4066.666_666_666_666_5).abs() < 1e-9);
        assert_eq!(frame_to_millis(CREDITS_RETRACT_START_FRAME), 6600.0);
        assert_eq!(frame_to_millis(CREDITS_GONE_FRAME), 7150.0);
    }

    #[test]
    fn canonical_badge_polygon_fits_measured_raster_extent() {
        for (x, y) in BADGE_CANONICAL_VERTICES {
            assert!(x >= 0 && x < BADGE_CANONICAL_RED_WIDTH as i32);
            assert!(y >= 0 && y < BADGE_CANONICAL_RED_HEIGHT as i32);
        }
        assert_eq!(BADGE_CANONICAL_VERTICES[0], (148, 0));
        assert_eq!(BADGE_CANONICAL_VERTICES[3], (151, 343));
    }

    #[test]
    fn shine_and_fade_windows_use_exact_source_frames() {
        assert_eq!(frame_to_millis(SECOND_BADGE_SHINE_STRONG_START_FRAME), 3900.0);
        assert_eq!(frame_to_millis(SECOND_BADGE_SHINE_STRONG_END_FRAME), 4150.0);
        assert_eq!(frame_to_millis(OUTRO_FADE_FIRST_FRAME), 203000.0);
        assert_eq!(frame_to_millis(OUTRO_BLACK_FIRST_FRAME), 204300.0);
        assert_eq!(sample_reference_frame(OUTRO_FADE_FIRST_FRAME - 1).stage, TimelineStage::Outro);
        assert_eq!(sample_reference_frame(OUTRO_FADE_FIRST_FRAME).stage, TimelineStage::Fade);
    }

    #[test]
    fn first_card_reveal_uses_measured_source_widths() {
        assert_eq!(first_card_reveal_width_px(4), 0);
        assert_eq!(first_card_reveal_width_px(5), 9);
        assert_eq!(first_card_reveal_width_px(30), 348);
        assert_eq!(first_card_reveal_width_px(81), 480);
        assert_eq!(first_card_reveal_width_px(500), 480);
    }

    #[test]
    fn intro_pan_uses_exact_half_pixel_track_and_continuous_cruise_anchor() {
        assert_eq!(measured_card_train_x_px(523), 0.0);
        assert_eq!(measured_card_train_x_px(524), -0.5);
        assert_eq!(measured_card_train_x_px(598), -208.5);
        assert_eq!(measured_card_train_x_px(630), -313.5);
        let next = measured_card_train_x_px(631);
        assert!((next - (-313.5 - ReferenceMotion::default().steady_scroll_px_per_frame)).abs() < 1e-12);
    }

    #[test]
    fn credits_exit_uses_measured_source_edges() {
        assert_eq!(credits_left_x_px(395), Some(1431));
        assert_eq!(credits_left_x_px(396), Some(1703));
        assert_eq!(credits_left_x_px(411), Some(1825));
        assert_eq!(credits_left_x_px(428), Some(1893));
        assert_eq!(credits_left_x_px(429), None);
    }
    #[test]
    fn shine_and_fade_use_consecutive_frame_fixtures() {
        assert!(second_badge_shine_sample(231).is_none());
        let shine = second_badge_shine_sample(234).unwrap();
        assert!((shine.normal_center_px - 142.862_32).abs() < 1e-4);
        assert!(second_badge_shine_sample(252).is_none());
        assert_eq!(outro_fade_level(12_179), 1.0);
        assert!((outro_fade_level(12_180) - 0.9904).abs() < 0.0001);
        assert_eq!(outro_fade_level(12_258), 0.0);
        assert_eq!(outro_fade_level(12_266), 0.0);
    }

}
