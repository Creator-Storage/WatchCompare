use serde::{Deserialize, Serialize};

pub const REFERENCE_WIDTH: u32 = 1920;
pub const REFERENCE_HEIGHT: u32 = 1080;
pub const REFERENCE_FPS: f64 = 60.0;
pub const REFERENCE_FRAME_COUNT: u64 = 12_267;
pub const REFERENCE_DURATION_SECONDS: f64 = 204.45;
pub const REFERENCE_TIME_BASE_DEN: u64 = 15_360;
pub const REFERENCE_TICKS_PER_FRAME: u64 = 256;
pub const REFERENCE_FRAME_DURATION_MILLIS: f64 = 50.0 / 3.0;

pub const BADGE_CANONICAL_RED_WIDTH: u32 = 298;
pub const BADGE_CANONICAL_RED_HEIGHT: u32 = 344;
pub const BADGE_CANONICAL_VERTICES: [(i32, i32); 6] = [
    (148, 0), (2, 84), (0, 255), (151, 343), (297, 257), (297, 84),
];

pub const FIRST_BADGE_FIRST_VISIBLE_FRAME: u64 = 34;
pub const SECOND_CARD_FIRST_VISIBLE_FRAME: u64 = 125;
pub const THIRD_CARD_FIRST_VISIBLE_FRAME: u64 = 244;
pub const CREDITS_RETRACT_START_FRAME: u64 = 396;
pub const CREDITS_GONE_FRAME: u64 = 429;
pub const INTRO_PAN_FIRST_MOVING_FRAME: u64 = 524;
pub const INTRO_PAN_SETTLED_FRAME: u64 = 630;

pub const SECOND_BADGE_SHINE_FAINT_START_FRAME: u64 = 232;
pub const SECOND_BADGE_SHINE_STRONG_START_FRAME: u64 = 234;
pub const SECOND_BADGE_SHINE_STRONG_END_FRAME: u64 = 249;
pub const SECOND_BADGE_SHINE_GONE_FRAME: u64 = 252;

pub const OUTRO_CARD_TRAIN_STOP_FRAME: u64 = 11_843;
pub const OUTRO_WIPE_START_FRAME: u64 = 11_868;
pub const OUTRO_WIPE_FULL_FRAME: u64 = 11_884;
pub const OUTRO_GROUP_START_FRAME: u64 = 11_901;
pub const OUTRO_GROUP_SETTLED_FRAME: u64 = 11_911;
pub const OUTRO_CTA_FIRST_VISIBLE_FRAME: u64 = 11_913;
pub const OUTRO_CTA_SETTLED_FRAME: u64 = 11_957;
pub const OUTRO_FADE_FIRST_FRAME: u64 = 12_180;
pub const OUTRO_BLACK_FIRST_FRAME: u64 = 12_258;

pub const FIRST_CARD_REVEAL_WIDTH_PX: [u16; 91] = [
    0, 0, 0, 0, 0, 9, 12, 17, 22, 29, 37, 47, 60, 75, 93, 114, 137, 160,
    183, 204, 224, 242, 258, 273, 286, 298, 310, 320, 330, 339, 348, 355, 363,
    370, 376, 382, 388, 394, 399, 404, 408, 413, 417, 421, 424, 428, 431, 434,
    437, 440, 443, 446, 448, 450, 453, 455, 457, 459, 460, 462, 464, 465, 467,
    468, 469, 470, 471, 472, 473, 474, 475, 476, 476, 477, 477, 478, 478, 479,
    479, 479, 479, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480,
];

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

pub const CREDITS_EDGE_TRACK_START_FRAME: u64 = 396;
pub const CREDITS_LEFT_X_PX: [u16; 33] = [
    1703, 1717, 1730, 1741, 1752, 1762, 1771, 1779, 1788, 1795, 1800, 1810,
    1816, 1822, 1826, 1825, 1838, 1843, 1848, 1852, 1856, 1860, 1864, 1868,
    1871, 1874, 1877, 1880, 1883, 1886, 1888, 1891, 1893,
];

pub const SECOND_BADGE_SHINE_TRACK_START_FRAME: u64 = 232;
pub const SECOND_BADGE_SHINE_NORMAL_CENTER_PX: [f32; 20] = [
    127.516, 134.185, 142.862, 151.458, 162.104, 170.791, 182.513, 201.631,
    217.594, 232.100, 255.732, 275.272, 297.427, 323.617, 343.087, 367.092,
    388.148, 409.323, 422.615, 441.565,
];
pub const SECOND_BADGE_SHINE_WIDTH80_PX: [f32; 20] = [
    12.2988, 15.6695, 22.9638, 29.5085, 37.0457, 37.4012, 39.9066, 41.3882,
    39.3963, 38.8319, 39.6867, 34.3059, 32.1609, 32.0696, 26.8857, 27.4303,
    20.9928, 16.1293, 37.0101, 16.6275,
];

pub const OUTRO_FADE_TRACK_START_FRAME: u64 = 12_179;
pub const OUTRO_FADE_LEVEL_10000: [u16; 80] = [
    10000, 9904, 9766, 9726, 9726, 9726, 9483, 9483, 9441, 9303, 9303, 9204,
    9204, 9204, 8876, 8876, 8738, 8455, 8411, 8416, 7986, 7986, 7986, 7932,
    7570, 7570, 7430, 7239, 7239, 7101, 6918, 6816, 6580, 6580, 6351, 6351,
    6157, 6157, 6017, 5833, 5643, 5599, 5360, 5360, 5360, 5223, 4753, 4753,
    4709, 4615, 4475, 4279, 4279, 4139, 3906, 3906, 3763, 3670, 3527, 3389,
    3296, 3204, 3066, 3013, 2777, 2777, 2639, 2499, 2116, 2261, 2030, 1976,
    1840, 1700, 1556, 1411, 1273, 1035, 705, 0,
];

pub const CARD_TRAIN_EXACT_TRACK_START_FRAME: u64 = 630;
pub const CARD_TRAIN_EXACT_TRACK_END_FRAME: u64 = OUTRO_CARD_TRAIN_STOP_FRAME;
pub const CARD_TRAIN_EXACT_BLOCK_FRAMES: usize = 64;
const CARD_TRAIN_DELTA_HALF_PX_DIGITS: &str = include_str!("reference_card_train_delta_half_px.txt");
pub const CARD_TRAIN_CHECKPOINT_HALF_PX: [i32; 176] = [
    -627, -908, -1192, -1474, -1762, -2043, -2326, -2612, -2895, -3180, -3465,
    -3749, -4033, -4316, -4601, -4887, -5170, -5455, -5738, -6023, -6309,
    -6592, -6877, -7160, -7446, -7731, -8014, -8299, -8582, -8868, -9153,
    -9436, -9719, -10001, -10289, -10573, -10855, -11141, -11426, -11707,
    -11993, -12275, -12561, -12846, -13130, -13415, -13698, -13983, -14267,
    -14552, -14837, -15121, -15405, -15689, -15975, -16260, -16543, -16828,
    -17111, -17397, -17682, -17965, -18250, -18533, -18819, -19104, -19387,
    -19672, -19955, -20241, -20526, -20809, -21094, -21377, -21663, -21948,
    -22231, -22517, -22800, -23085, -23370, -23653, -23939, -24222, -24507,
    -24792, -25075, -25361, -25644, -25929, -26214, -26498, -26783, -27066,
    -27351, -27637, -27920, -28205, -28488, -28774, -29059, -29342, -29627,
    -29911, -30196, -30481, -30765, -31049, -31333, -31619, -31903, -32187,
    -32471, -32755, -33041, -33326, -33609, -33894, -34177, -34463, -34748,
    -35031, -35316, -35599, -35885, -36170, -36453, -36738, -37021, -37307,
    -37592, -37875, -38160, -38443, -38728, -39013, -39295, -39580, -39865,
    -40149, -40434, -40718, -41002, -41286, -41571, -41856, -42139, -42424,
    -42708, -42996, -43281, -43565, -43849, -44133, -44418, -44704, -44986,
    -45270, -45555, -45839, -46124, -46407, -46692, -46976, -47262, -47546,
    -47829, -48114, -48398, -48683, -48968, -49251, -49536, -49819, -50105,
    -50390,
];

pub const OUTRO_WIPE_BOTTOM_Y: [u16; 17] = [
    23, 67, 125, 191, 267, 351, 439, 531, 621, 709, 792, 871, 939, 996, 1039,
    1067, 1079,
];
pub const OUTRO_PANEL_TOP_Y: [i16; 11] = [-60, -14, 27, 66, 102, 132, 159, 180, 196, 206, 210];
pub const OUTRO_CREDITS_TOP_Y: [u16; 11] = [470, 515, 557, 596, 632, 663, 690, 711, 727, 737, 740];
pub const OUTRO_CTA_BBOX: [[u16; 4]; 45] = [
    [696,93,82,18],[696,93,82,18],[665,85,144,33],[665,85,144,33],[632,77,211,50],
    [632,77,211,50],[580,64,314,75],[580,64,314,75],[562,60,350,84],[562,60,351,84],
    [548,56,379,91],[548,56,379,91],[548,56,379,91],[536,53,403,97],[536,53,403,97],
    [517,49,441,106],[517,49,441,106],[510,47,455,109],[510,47,455,109],[503,45,469,113],
    [503,45,469,113],[497,44,480,116],[497,44,480,116],[488,41,499,121],[488,41,499,121],
    [485,40,505,123],[485,41,505,122],[481,40,513,124],[481,40,513,124],[478,39,519,125],
    [478,39,519,125],[474,38,527,127],[474,38,527,127],[472,38,531,128],[472,38,531,128],
    [471,37,533,129],[471,37,533,129],[471,37,533,129],[470,37,535,129],[470,37,535,129],
    [468,36,539,131],[468,36,539,131],[467,36,540,131],[467,36,540,131],[467,36,541,131],
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ShineSample { pub normal_center_px: f32, pub width80_px: f32 }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OutroGroupSample { pub panel_top_y: i16, pub credits_top_y: u16 }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RectU16 { pub x: u16, pub y: u16, pub width: u16, pub height: u16 }

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
        Self { card_pitch_px: 477.0, artwork_bottom_y: 871, title_top_y: 872,
            title_bottom_y: 964, description_top_y: 965, description_bottom_y: 1074,
            bottom_border_top_y: 1075, separator_nominal_px: 6.0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ReferenceMotion { pub steady_scroll_px_per_second: f64, pub steady_scroll_px_per_frame: f64 }
impl Default for ReferenceMotion {
    fn default() -> Self {
        let px_per_second = 133.473_127_893_786_43;
        Self { steady_scroll_px_per_second: px_per_second, steady_scroll_px_per_frame: px_per_second / REFERENCE_FPS }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimelineStage { Intro, Cruise, Outro, Fade }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceProfile {
    pub width: u32, pub height: u32, pub fps: f64, pub frame_count: u64,
    pub duration_seconds: f64, pub geometry: ReferenceGeometry, pub motion: ReferenceMotion,
    pub status: &'static str,
}
impl Default for ReferenceProfile {
    fn default() -> Self {
        Self { width: REFERENCE_WIDTH, height: REFERENCE_HEIGHT, fps: REFERENCE_FPS,
            frame_count: REFERENCE_FRAME_COUNT, duration_seconds: REFERENCE_DURATION_SECONDS,
            geometry: ReferenceGeometry::default(), motion: ReferenceMotion::default(),
            status: "measurement_pass_4_exact_cruise_and_outro_build" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameState {
    pub frame: u64, pub pts_ticks: u64, pub time_millis: f64, pub time_seconds: f64,
    pub stage: TimelineStage, pub card_train_x_px: f64, pub card_phase_px: f64,
}

pub const fn frame_to_pts_ticks(frame: u64) -> u64 { frame * REFERENCE_TICKS_PER_FRAME }
pub fn pts_ticks_to_seconds(ticks: u64) -> f64 { ticks as f64 / REFERENCE_TIME_BASE_DEN as f64 }
pub fn pts_ticks_to_millis(ticks: u64) -> f64 { ticks as f64 * 1000.0 / REFERENCE_TIME_BASE_DEN as f64 }
pub fn frame_to_seconds(frame: u64) -> f64 { pts_ticks_to_seconds(frame_to_pts_ticks(frame)) }
pub fn frame_to_millis(frame: u64) -> f64 { pts_ticks_to_millis(frame_to_pts_ticks(frame)) }
pub fn seconds_to_frame(seconds: f64) -> u64 { (seconds.max(0.0) * REFERENCE_FPS).round() as u64 }

pub fn first_card_reveal_width_px(frame: u64) -> u16 {
    FIRST_CARD_REVEAL_WIDTH_PX.get(frame as usize).copied().unwrap_or(480)
}

pub fn measured_card_train_x_px(frame: u64) -> f64 {
    if frame < CARD_TRAIN_EXACT_TRACK_START_FRAME {
        if frame <= INTRO_PAN_TRACK_START_FRAME { return 0.0; }
        return INTRO_PAN_HALF_PX[(frame - INTRO_PAN_TRACK_START_FRAME) as usize] as f64 / 2.0;
    }
    let sampled_frame = frame.min(CARD_TRAIN_EXACT_TRACK_END_FRAME);
    let offset = (sampled_frame - CARD_TRAIN_EXACT_TRACK_START_FRAME) as usize;
    let block = offset / CARD_TRAIN_EXACT_BLOCK_FRAMES;
    let within_block = offset % CARD_TRAIN_EXACT_BLOCK_FRAMES;
    let mut half_px = CARD_TRAIN_CHECKPOINT_HALF_PX[block];
    let first_delta = block * CARD_TRAIN_EXACT_BLOCK_FRAMES;
    let digits = CARD_TRAIN_DELTA_HALF_PX_DIGITS.as_bytes();
    for digit in &digits[first_delta..first_delta + within_block] { half_px -= (digit - b'0') as i32; }
    half_px as f64 / 2.0
}

pub fn credits_left_x_px(frame: u64) -> Option<u16> {
    if frame < CREDITS_RETRACT_START_FRAME { return Some(1431); }
    if frame >= CREDITS_GONE_FRAME { return None; }
    CREDITS_LEFT_X_PX.get((frame - CREDITS_EDGE_TRACK_START_FRAME) as usize).copied()
}

pub fn outro_wipe_bottom_y(frame: u64) -> Option<u16> {
    if frame < OUTRO_WIPE_START_FRAME { return None; }
    Some(*OUTRO_WIPE_BOTTOM_Y.get((frame - OUTRO_WIPE_START_FRAME) as usize).unwrap_or(&1079))
}

pub fn outro_group_sample(frame: u64) -> Option<OutroGroupSample> {
    if frame < OUTRO_GROUP_START_FRAME { return None; }
    let i = ((frame - OUTRO_GROUP_START_FRAME) as usize).min(OUTRO_PANEL_TOP_Y.len() - 1);
    Some(OutroGroupSample { panel_top_y: OUTRO_PANEL_TOP_Y[i], credits_top_y: OUTRO_CREDITS_TOP_Y[i] })
}

pub fn outro_cta_bbox(frame: u64) -> Option<RectU16> {
    if frame < OUTRO_CTA_FIRST_VISIBLE_FRAME { return None; }
    let i = ((frame - OUTRO_CTA_FIRST_VISIBLE_FRAME) as usize).min(OUTRO_CTA_BBOX.len() - 1);
    let [x, y, width, height] = OUTRO_CTA_BBOX[i];
    Some(RectU16 { x, y, width, height })
}

pub fn second_badge_shine_sample(frame: u64) -> Option<ShineSample> {
    if !(SECOND_BADGE_SHINE_TRACK_START_FRAME..SECOND_BADGE_SHINE_GONE_FRAME).contains(&frame) { return None; }
    let i = (frame - SECOND_BADGE_SHINE_TRACK_START_FRAME) as usize;
    Some(ShineSample { normal_center_px: SECOND_BADGE_SHINE_NORMAL_CENTER_PX[i], width80_px: SECOND_BADGE_SHINE_WIDTH80_PX[i] })
}

pub fn outro_fade_level(frame: u64) -> f64 {
    if frame < OUTRO_FADE_TRACK_START_FRAME { return 1.0; }
    OUTRO_FADE_LEVEL_10000.get((frame - OUTRO_FADE_TRACK_START_FRAME) as usize)
        .map(|v| *v as f64 / 10_000.0).unwrap_or(0.0)
}

pub fn sample_reference_frame(frame: u64) -> FrameState {
    let profile = ReferenceProfile::default();
    let frame = frame.min(REFERENCE_FRAME_COUNT.saturating_sub(1));
    let pts_ticks = frame_to_pts_ticks(frame);
    let time_seconds = pts_ticks_to_seconds(pts_ticks);
    let time_millis = pts_ticks_to_millis(pts_ticks);
    let stage = if frame < INTRO_PAN_SETTLED_FRAME { TimelineStage::Intro }
        else if frame < OUTRO_CARD_TRAIN_STOP_FRAME { TimelineStage::Cruise }
        else if frame < OUTRO_FADE_FIRST_FRAME { TimelineStage::Outro }
        else { TimelineStage::Fade };
    let x = measured_card_train_x_px(frame);
    let pitch = profile.geometry.card_pitch_px;
    let card_phase_px = ((x % pitch) + pitch) % pitch;
    FrameState { frame, pts_ticks, time_millis, time_seconds, stage, card_train_x_px: x, card_phase_px }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn geometry() {
        let g = ReferenceGeometry::default();
        assert_eq!(g.title_top_y, g.artwork_bottom_y + 1);
        assert_eq!(g.description_top_y, g.title_bottom_y + 1);
        assert_eq!(g.bottom_border_top_y, g.description_bottom_y + 1);
    }
    #[test] fn exact_clock() {
        assert_eq!(frame_to_pts_ticks(1), 256);
        assert!((frame_to_millis(1) - 50.0 / 3.0).abs() < 1e-12);
        assert_eq!(frame_to_millis(60), 1000.0);
    }
    #[test] fn exact_card_train() {
        assert_eq!(CARD_TRAIN_DELTA_HALF_PX_DIGITS.len(), 11_213);
        assert_eq!(measured_card_train_x_px(523), 0.0);
        assert_eq!(measured_card_train_x_px(524), -0.5);
        assert_eq!(measured_card_train_x_px(598), -208.5);
        assert_eq!(measured_card_train_x_px(630), -313.5);
        assert_eq!(measured_card_train_x_px(1200), -1576.5);
        assert_eq!(measured_card_train_x_px(1230), -1643.5);
        assert_eq!(measured_card_train_x_px(11_842), -25_221.0);
        assert_eq!(measured_card_train_x_px(11_843), -25_221.0);
        assert_eq!(measured_card_train_x_px(12_000), -25_221.0);
    }
    #[test] fn exact_intro_and_credits() {
        assert_eq!(first_card_reveal_width_px(5), 9);
        assert_eq!(first_card_reveal_width_px(81), 480);
        assert_eq!(credits_left_x_px(395), Some(1431));
        assert_eq!(credits_left_x_px(396), Some(1703));
        assert_eq!(credits_left_x_px(428), Some(1893));
        assert_eq!(credits_left_x_px(429), None);
    }
    #[test] fn exact_shine_and_fade() {
        let shine = second_badge_shine_sample(234).unwrap();
        assert!((shine.normal_center_px - 142.86232).abs() < 1e-4);
        assert!(second_badge_shine_sample(252).is_none());
        assert_eq!(outro_fade_level(12_179), 1.0);
        assert!((outro_fade_level(12_180) - 0.9904).abs() < 0.0001);
        assert_eq!(outro_fade_level(12_258), 0.0);
    }
    #[test] fn exact_outro_build() {
        assert_eq!(sample_reference_frame(11_842).stage, TimelineStage::Cruise);
        assert_eq!(sample_reference_frame(11_843).stage, TimelineStage::Outro);
        assert_eq!(outro_wipe_bottom_y(11_867), None);
        assert_eq!(outro_wipe_bottom_y(11_868), Some(23));
        assert_eq!(outro_wipe_bottom_y(11_884), Some(1079));
        assert_eq!(outro_group_sample(11_901).unwrap().panel_top_y, -60);
        assert_eq!(outro_group_sample(11_911).unwrap(), OutroGroupSample { panel_top_y: 210, credits_top_y: 740 });
        assert_eq!(outro_cta_bbox(11_913).unwrap(), RectU16 { x: 696, y: 93, width: 82, height: 18 });
        assert_eq!(outro_cta_bbox(11_957).unwrap(), RectU16 { x: 467, y: 36, width: 541, height: 131 });
    }
}
