use watchcompare_render::{sample_reference_frame, FrameState, ReferenceProfile};

#[tauri::command]
fn reference_profile() -> ReferenceProfile {
    ReferenceProfile::default()
}

#[tauri::command]
fn sample_reference(frame: u64) -> FrameState {
    sample_reference_frame(frame)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![reference_profile, sample_reference])
        .run(tauri::generate_context!())
        .expect("error while running WatchCompare");
}
