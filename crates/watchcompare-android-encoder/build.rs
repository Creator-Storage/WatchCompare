const COMMANDS: &[&str] = &["begin", "pushFrame", "finish", "cancel"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .try_build()
        .expect("failed to build WatchCompare Android encoder plugin");
}
