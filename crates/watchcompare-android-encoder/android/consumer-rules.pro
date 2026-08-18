# WatchCompare encoder uses only Android framework media APIs and Tauri plugin annotations.
-keep @app.tauri.annotation.TauriPlugin class * { *; }
-keepclassmembers class * {
    @app.tauri.annotation.Command <methods>;
}
