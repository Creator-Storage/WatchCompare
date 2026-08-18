#![cfg(target_os = "android")]

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

const PLUGIN_IDENTIFIER: &str = "network.creative.watchcompare.encoder";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginRequest {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    pub frame_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushFrameRequest {
    pub path: String,
    pub frame_index: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishRequest {
    pub soundtrack_path: Option<String>,
    pub audio_bitrate: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishResponse {
    pub path: String,
    pub video_codec: String,
    pub audio_codec: Option<String>,
}

pub struct AndroidEncoder<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> AndroidEncoder<R> {
    pub fn begin(&self, request: BeginRequest) -> Result<(), String> {
        self.0
            .run_mobile_plugin("begin", request)
            .map_err(|e| e.to_string())
    }

    pub fn push_frame(&self, request: PushFrameRequest) -> Result<(), String> {
        self.0
            .run_mobile_plugin("pushFrame", request)
            .map_err(|e| e.to_string())
    }

    pub fn finish(&self, request: FinishRequest) -> Result<FinishResponse, String> {
        self.0
            .run_mobile_plugin("finish", request)
            .map_err(|e| e.to_string())
    }

    pub fn cancel(&self) -> Result<(), String> {
        self.0
            .run_mobile_plugin("cancel", ())
            .map_err(|e| e.to_string())
    }
}

pub trait AndroidEncoderExt<R: Runtime> {
    fn android_encoder(&self) -> &AndroidEncoder<R>;
}

impl<R: Runtime, T: Manager<R>> AndroidEncoderExt<R> for T {
    fn android_encoder(&self) -> &AndroidEncoder<R> {
        self.state::<AndroidEncoder<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("watchcompare-encoder")
        .setup(|app, api| {
            let handle = api.register_android_plugin(
                PLUGIN_IDENTIFIER,
                "WatchCompareEncoderPlugin",
            )?;
            app.manage(AndroidEncoder(handle));
            Ok(())
        })
        .build()
}
