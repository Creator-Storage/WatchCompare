use serde::Serialize;
use watchcompare_fixtures::{
    cta_bell_fill_metrics, cta_like_blue_level, cta_subscribed_bbox,
    second_badge_text_white_area_px, second_badge_transform, BadgeTransform,
    BellFillMetrics, Rect,
};
use watchcompare_render::{
    credits_left_x_px, first_card_reveal_width_px, outro_cta_bbox, outro_fade_level,
    outro_group_sample, outro_wipe_bottom_y, sample_reference_frame,
    second_badge_shine_sample, FrameState, OutroGroupSample, RectU16, ReferenceProfile,
    ShineSample, REFERENCE_FRAME_COUNT,
};

#[derive(Debug, Clone, Serialize)]
struct SceneState {
    frame_state: FrameState,
    first_card_reveal_width_px: u16,
    credits_left_x_px: Option<u16>,
    second_badge_transform: Option<BadgeTransform>,
    second_badge_text_white_area_px: Option<u16>,
    second_badge_shine: Option<ShineSample>,
    outro_wipe_bottom_y: Option<u16>,
    outro_group: Option<OutroGroupSample>,
    outro_cta_bbox: Option<RectU16>,
    outro_fade_level: f64,
    cta_like_blue_level: Option<f32>,
    cta_subscribed_bbox: Option<Rect>,
    cta_bell_fill: Option<BellFillMetrics>,
}

fn scene_state(frame: u64) -> SceneState {
    let frame = frame.min(REFERENCE_FRAME_COUNT.saturating_sub(1));
    SceneState {
        frame_state: sample_reference_frame(frame),
        first_card_reveal_width_px: first_card_reveal_width_px(frame),
        credits_left_x_px: credits_left_x_px(frame),
        second_badge_transform: second_badge_transform(frame),
        second_badge_text_white_area_px: second_badge_text_white_area_px(frame),
        second_badge_shine: second_badge_shine_sample(frame),
        outro_wipe_bottom_y: outro_wipe_bottom_y(frame),
        outro_group: outro_group_sample(frame),
        outro_cta_bbox: outro_cta_bbox(frame),
        outro_fade_level: outro_fade_level(frame),
        cta_like_blue_level: cta_like_blue_level(frame),
        cta_subscribed_bbox: cta_subscribed_bbox(frame),
        cta_bell_fill: cta_bell_fill_metrics(frame),
    }
}

#[tauri::command]
fn reference_profile() -> ReferenceProfile {
    ReferenceProfile::default()
}

#[tauri::command]
fn sample_reference(frame: u64) -> FrameState {
    sample_reference_frame(frame)
}

#[tauri::command]
fn reference_scene(frame: u64) -> SceneState {
    scene_state(frame)
}

#[tauri::command]
fn reference_track() -> Vec<FrameState> {
    (0..REFERENCE_FRAME_COUNT).map(sample_reference_frame).collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            reference_profile,
            sample_reference,
            reference_scene,
            reference_track
        ])
        .run(tauri::generate_context!())
        .expect("error while running WatchCompare");
}
