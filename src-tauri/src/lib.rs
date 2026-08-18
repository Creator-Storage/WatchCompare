use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use watchcompare_compositor::{load_font_file, render_project_png};
use watchcompare_export::{export_mp4, ExportProgress};
use watchcompare_project::Project;
use watchcompare_render::{sample_reference_frame, FrameState, ReferenceProfile};
use watchcompare_scene::{sample_reference_scene, ReferenceSceneState};

#[cfg(target_os = "android")]
use watchcompare_android_encoder::{
    AndroidEncoderExt, BeginRequest as AndroidBeginRequest,
    FinishRequest as AndroidFinishRequest, PushFrameRequest as AndroidPushFrameRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub mobile: bool,
    pub mp4_encoder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJobStatus {
    pub id: String,
    pub stage: String,
    pub fraction: f32,
    pub completed_frames: u64,
    pub total_frames: u64,
    pub output_path: String,
    pub done: bool,
    pub cancelled: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
struct ExportJob {
    status: Arc<Mutex<ExportJobStatus>>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
struct AppState {
    jobs: Mutex<HashMap<String, ExportJob>>,
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
fn sample_reference_scene_state(frame: u64) -> ReferenceSceneState {
    sample_reference_scene(frame)
}

#[tauri::command]
fn platform_info() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.into(),
        mobile: cfg!(target_os = "android") || cfg!(target_os = "ios"),
        mp4_encoder: if cfg!(target_os = "android") {
            "android-mediacodec-h264"
        } else {
            "ffmpeg-libx264"
        }
        .into(),
    }
}

#[tauri::command]
fn new_project() -> Project {
    Project::demo()
}

#[tauri::command]
fn validate_project(project: Project) -> Result<(), String> {
    project.validate()
}

#[tauri::command]
fn render_preview(mut project: Project, frame: u64) -> Result<String, String> {
    project.validate()?;
    let explicit_font = project.font_path.clone();
    let fallback = default_platform_font();
    let chosen = explicit_font.or_else(|| fallback.map(|p| p.to_string_lossy().into_owned()));
    project.font_path = chosen.clone();
    let font = chosen.as_deref().and_then(|path| load_font_file(path).ok());
    let png = render_project_png(&project, frame, font.as_ref())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

#[tauri::command]
fn save_project(path: String, project: Project) -> Result<(), String> {
    project.validate()?;
    let json = serde_json::to_string_pretty(&project).map_err(|e| format!("serialize project: {e}"))?;
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    std::fs::write(&path, json).map_err(|e| format!("write {}: {e}", path.display()))
}

#[tauri::command]
fn load_project(path: String) -> Result<Project, String> {
    let path = PathBuf::from(path);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let project: Project = serde_json::from_str(&text).map_err(|e| format!("parse project: {e}"))?;
    project.validate()?;
    Ok(project)
}

#[tauri::command]
fn suggested_project_path(project: Project) -> String {
    default_output_dir()
        .join(format!("{}.watchcompare.json", safe_name(&project.name)))
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
fn suggested_export_path(project: Project) -> String {
    default_output_dir()
        .join(format!("{}.mp4", safe_name(&project.name)))
        .to_string_lossy()
        .into_owned()
}

#[cfg(target_os = "android")]
fn export_android_mp4<R, F, C>(
    app: &tauri::AppHandle<R>,
    project: &Project,
    scratch_root: &Path,
    output: &Path,
    mut progress: F,
    mut cancelled: C,
) -> Result<(), String>
where
    R: tauri::Runtime,
    F: FnMut(ExportProgress),
    C: FnMut() -> bool,
{
    project.validate()?;
    std::fs::create_dir_all(scratch_root)
        .map_err(|e| format!("create {}: {e}", scratch_root.display()))?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {}: {e}", parent.display()))?;
    }

    let frame_path = scratch_root.join(format!("watchcompare-frame-{}.png", timestamp_nanos()));
    let output_path = output.to_string_lossy().into_owned();
    let total = project.duration_frames();
    let encoder = app.android_encoder();
    encoder.begin(AndroidBeginRequest {
        output_path,
        width: project.export.width,
        height: project.export.height,
        fps: project.export.fps,
        bitrate: project.export.video_bitrate_mbps.max(1) * 1_000_000,
        frame_count: total,
    })?;

    let font = project
        .font_path
        .as_deref()
        .and_then(|path| load_font_file(path).ok());
    let report_every = (project.export.fps.max(1) as u64 / 4).max(1);

    let result = (|| {
        for frame in 0..total {
            if cancelled() {
                let _ = encoder.cancel();
                return Err("export cancelled".into());
            }

            let png = render_project_png(project, frame, font.as_ref())?;
            std::fs::write(&frame_path, png)
                .map_err(|e| format!("write {}: {e}", frame_path.display()))?;
            encoder.push_frame(AndroidPushFrameRequest {
                path: frame_path.to_string_lossy().into_owned(),
                frame_index: frame,
            })?;

            if frame == 0 || frame + 1 == total || (frame + 1) % report_every == 0 {
                progress(ExportProgress {
                    completed_frames: frame + 1,
                    total_frames: total,
                    fraction: (frame + 1) as f32 / total.max(1) as f32,
                    stage: "Rendering + hardware encoding".into(),
                });
            }
        }

        if cancelled() {
            let _ = encoder.cancel();
            return Err("export cancelled".into());
        }

        progress(ExportProgress {
            completed_frames: total,
            total_frames: total,
            fraction: 0.995,
            stage: "Finalizing MP4".into(),
        });
        encoder.finish(AndroidFinishRequest {
            soundtrack_path: project.export.soundtrack_path.clone(),
            audio_bitrate: project.export.audio_bitrate_kbps.max(64) * 1_000,
        })?;
        progress(ExportProgress {
            completed_frames: total,
            total_frames: total,
            fraction: 1.0,
            stage: "Complete".into(),
        });
        Ok(())
    })();

    let _ = std::fs::remove_file(&frame_path);
    if result.is_err() {
        let _ = encoder.cancel();
    }
    result
}

#[tauri::command]
fn export_start(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mut project: Project,
    output_path: String,
) -> Result<String, String> {
    project.validate()?;
    if project.font_path.is_none() {
        project.font_path = default_platform_font().map(|p| p.to_string_lossy().into_owned());
    }

    let id = format!("job-{}", timestamp_nanos());
    let status = Arc::new(Mutex::new(ExportJobStatus {
        id: id.clone(),
        stage: "Queued".into(),
        fraction: 0.0,
        completed_frames: 0,
        total_frames: project.duration_frames(),
        output_path: output_path.clone(),
        done: false,
        cancelled: false,
        error: None,
    }));
    let cancel = Arc::new(AtomicBool::new(false));
    state.jobs.lock().map_err(|_| "export job lock poisoned")?.insert(
        id.clone(),
        ExportJob {
            status: status.clone(),
            cancel: cancel.clone(),
        },
    );

    std::thread::spawn(move || {
        let update = |progress: ExportProgress| {
            if let Ok(mut current) = status.lock() {
                current.stage = progress.stage;
                current.fraction = progress.fraction;
                current.completed_frames = progress.completed_frames;
                current.total_frames = progress.total_frames;
            }
        };
        let scratch = std::env::temp_dir().join("watchcompare-export");
        let _ = std::fs::create_dir_all(&scratch);
        let output = PathBuf::from(&output_path);

        #[cfg(target_os = "android")]
        let result = export_android_mp4(
            &app,
            &project,
            &scratch,
            &output,
            update,
            || cancel.load(Ordering::Relaxed),
        );

        #[cfg(not(target_os = "android"))]
        let result = export_mp4(
            &project,
            &scratch,
            &output,
            update,
            || cancel.load(Ordering::Relaxed),
        );

        if let Ok(mut current) = status.lock() {
            current.done = true;
            current.cancelled = cancel.load(Ordering::Relaxed)
                || matches!(result.as_deref(), Err("export cancelled"));
            match result {
                Ok(()) => {
                    current.fraction = 1.0;
                    current.stage = if current.cancelled {
                        "Cancelled".into()
                    } else {
                        "Complete".into()
                    };
                }
                Err(error) => {
                    current.stage = if current.cancelled {
                        "Cancelled".into()
                    } else {
                        "Failed".into()
                    };
                    if !current.cancelled {
                        current.error = Some(error);
                    }
                }
            }
        }
    });

    Ok(id)
}

#[tauri::command]
fn export_status(state: State<'_, AppState>, id: String) -> Result<Option<ExportJobStatus>, String> {
    let jobs = state.jobs.lock().map_err(|_| "export job lock poisoned")?;
    Ok(jobs
        .get(&id)
        .and_then(|job| job.status.lock().ok().map(|status| status.clone())))
}

#[tauri::command]
fn export_cancel(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let jobs = state.jobs.lock().map_err(|_| "export job lock poisoned")?;
    let Some(job) = jobs.get(&id) else {
        return Ok(false);
    };
    job.cancel.store(true, Ordering::Relaxed);
    if let Ok(mut status) = job.status.lock() {
        status.stage = "Cancelling".into();
    }
    Ok(true)
}

fn safe_name(name: &str) -> String {
    let mut result = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    while result.contains("--") {
        result = result.replace("--", "-");
    }
    let result = result.trim_matches('-').to_string();
    if result.is_empty() {
        "watchcompare".into()
    } else {
        result
    }
}

fn default_output_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir())
}

fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_nanos())
        .unwrap_or(0)
}

fn first_existing(paths: &[&str]) -> Option<PathBuf> {
    paths
        .iter()
        .map(Path::new)
        .find(|path| path.exists())
        .map(Path::to_path_buf)
}

fn default_platform_font() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return first_existing(&[
            r"C:\Windows\Fonts\seguisb.ttf",
            r"C:\Windows\Fonts\segoeui.ttf",
        ]);
    }
    #[cfg(target_os = "android")]
    {
        return first_existing(&[
            "/system/fonts/Roboto-Medium.ttf",
            "/system/fonts/Roboto-Regular.ttf",
        ]);
    }
    #[cfg(target_os = "linux")]
    {
        return first_existing(&[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]);
    }
    #[cfg(target_os = "macos")]
    {
        return first_existing(&[
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ]);
    }
    #[allow(unreachable_code)]
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().manage(AppState::default());

    #[cfg(target_os = "android")]
    let builder = builder.plugin(watchcompare_android_encoder::init());

    builder
        .invoke_handler(tauri::generate_handler![
            reference_profile,
            sample_reference,
            sample_reference_scene_state,
            platform_info,
            new_project,
            validate_project,
            render_preview,
            save_project,
            load_project,
            suggested_project_path,
            suggested_export_path,
            export_start,
            export_status,
            export_cancel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WatchCompare");
}
