mod megapack;
mod table_import;

use serde_json::{json, Value};
use watchcompare_render::{sample_reference_frame, FrameState, ReferenceProfile, REFERENCE_FRAME_COUNT};
use watchcompare_scene::sample_reference_scene;

#[tauri::command]
fn reference_profile() -> ReferenceProfile {
    ReferenceProfile::default()
}

#[tauri::command]
fn sample_reference(frame: u64) -> FrameState {
    sample_reference_frame(frame)
}

#[tauri::command]
fn reference_scene(frame: u64) -> Value {
    let scene = sample_reference_scene(frame);
    json!({
        "frame_state": sample_reference_frame(scene.frame),
        "first_card_reveal_width_px": scene.first_card_reveal_width_px,
        "credits_left_x_px": scene.credits_left_x_px,
        "second_badge_transform": scene.second_badge.transform,
        "second_badge_text_reveal_level": scene.second_badge.text_reveal_level,
        "second_badge_shine": scene.second_badge.shine,
        "mid_video_cta": scene.mid_video_cta,
        "outro_wipe_bottom_y": scene.outro.wipe_bottom_y,
        "outro_group": scene.outro.group,
        "outro_cta_bbox": scene.cta.outer_bbox,
        "outro_fade_level": scene.outro.fade_level,
        "cta_like_blue_level": scene.cta.like_blue_level,
        "cta_subscribed": scene.cta.subscribed,
        "cta_subscribed_bbox": scene.cta.subscribed_bbox,
        "cta_bell_filled": scene.cta.bell_filled,
        "cta_bell_fill_level": scene.cta.bell_fill_level,
        "cta_cursor_visible": scene.cta.cursor_visible,
        "cta_cursor_x_px": scene.cta.cursor_x_px,
        "cta_cursor_y_px": scene.cta.cursor_y_px
    })
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
            reference_track,
            megapack::import_megapack,
            megapack::export_megapack,
            table_import::import_table
        ])
        .run(tauri::generate_context!())
        .expect("error while running WatchCompare");
}
