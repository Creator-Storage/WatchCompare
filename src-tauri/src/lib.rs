use watchcompare_render::{sample_reference_frame, FrameState, ReferenceProfile, REFERENCE_FRAME_COUNT};
use watchcompare_scene::{sample_reference_scene, ReferenceSceneState};

#[tauri::command]
fn reference_profile() -> ReferenceProfile {
    ReferenceProfile::default()
}

#[tauri::command]
fn sample_reference(frame: u64) -> FrameState {
    sample_reference_frame(frame)
}

#[tauri::command]
fn reference_scene(frame: u64) -> ReferenceSceneState {
    sample_reference_scene(frame)
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
