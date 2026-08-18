use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use watchcompare_compositor::{load_font_file, render_project_frame};
use watchcompare_project::Project;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportProgress {
    pub completed_frames: u64,
    pub total_frames: u64,
    pub fraction: f32,
    pub stage: String,
}

impl ExportProgress {
    fn render(frame: u64, total: u64) -> Self {
        Self {
            completed_frames: frame,
            total_frames: total,
            fraction: if total == 0 { 0.0 } else { frame as f32 / total as f32 },
            stage: "Rendering frames".into(),
        }
    }

    fn encoding(total: u64) -> Self {
        Self { completed_frames: total, total_frames: total, fraction: 1.0, stage: "Encoding MP4".into() }
    }
}

pub fn render_png_sequence<F>(project: &Project, directory: &Path, mut progress: F) -> Result<(), String>
where
    F: FnMut(ExportProgress),
{
    project.validate()?;
    fs::create_dir_all(directory).map_err(|e| format!("create {}: {e}", directory.display()))?;
    let font = project
        .font_path
        .as_deref()
        .and_then(|path| load_font_file(path).ok());
    let total = project.duration_frames();
    for frame in 0..total {
        let image = render_project_frame(project, frame, font.as_ref())?;
        let path = directory.join(format!("frame-{frame:08}.png"));
        image.save(&path).map_err(|e| format!("save {}: {e}", path.display()))?;
        if frame == 0 || frame + 1 == total || frame % project.export.fps.max(1) as u64 == 0 {
            progress(ExportProgress::render(frame + 1, total));
        }
    }
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn encode_mp4_with_ffmpeg<F>(project: &Project, frames: &Path, output: &Path, mut progress: F) -> Result<(), String>
where
    F: FnMut(ExportProgress),
{
    use std::process::Command;
    let total = project.duration_frames();
    progress(ExportProgress::encoding(total));
    let input = frames.join("frame-%08d.png");
    let mut command = Command::new("ffmpeg");
    command
        .arg("-y")
        .arg("-framerate").arg(project.export.fps.to_string())
        .arg("-i").arg(&input)
        .arg("-c:v").arg("libx264")
        .arg("-preset").arg("medium")
        .arg("-pix_fmt").arg("yuv420p")
        .arg("-b:v").arg(format!("{}M", project.export.video_bitrate_mbps.max(1)));
    if let Some(audio) = project.export.soundtrack_path.as_deref() {
        command
            .arg("-i").arg(audio)
            .arg("-shortest")
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg(format!("{}k", project.export.audio_bitrate_kbps.max(64)));
    }
    let status = command.arg(output).status().map_err(|e| format!("launch ffmpeg: {e}"))?;
    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    Ok(())
}

#[cfg(target_os = "android")]
pub fn encode_mp4_with_ffmpeg<F>(_project: &Project, _frames: &Path, _output: &Path, _progress: F) -> Result<(), String>
where
    F: FnMut(ExportProgress),
{
    Err("MP4 encoding requires the Android media encoder bridge; frame rendering is available on Android".into())
}

pub fn export_mp4<F>(project: &Project, scratch_root: &Path, output: &Path, mut progress: F) -> Result<(), String>
where
    F: FnMut(ExportProgress),
{
    let frames = unique_frames_dir(scratch_root);
    render_png_sequence(project, &frames, |p| progress(p))?;
    let result = encode_mp4_with_ffmpeg(project, &frames, output, |p| progress(p));
    let _ = fs::remove_dir_all(&frames);
    result
}

fn unique_frames_dir(root: &Path) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_nanos())
        .unwrap_or(0);
    root.join(format!("watchcompare-{stamp}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_fraction_is_bounded() {
        let p = ExportProgress::render(30, 60);
        assert_eq!(p.fraction, 0.5);
        assert_eq!(ExportProgress::encoding(60).fraction, 1.0);
    }
}
