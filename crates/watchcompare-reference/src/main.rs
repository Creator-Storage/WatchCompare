use serde::Deserialize;
use std::{
    env, fs,
    path::Path,
    process::{Command, ExitCode},
};

#[derive(Debug, Deserialize)]
struct ProbeOutput {
    streams: Vec<ProbeStream>,
}

#[derive(Debug, Deserialize)]
struct ProbeStream {
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    duration: Option<String>,
    nb_frames: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("watchcompare-reference: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("probe") => {
            let input = args
                .next()
                .ok_or("usage: watchcompare-reference probe <video>")?;
            probe(&input)
        }
        Some("contact-sheet") => {
            let input = args.next().ok_or(
                "usage: watchcompare-reference contact-sheet <video> <out.jpg> [seconds-per-frame]",
            )?;
            let output = args.next().ok_or(
                "usage: watchcompare-reference contact-sheet <video> <out.jpg> [seconds-per-frame]",
            )?;
            let interval = args.next().unwrap_or_else(|| "4".into());
            contact_sheet(&input, &output, &interval)
        }
        _ => Err("commands: probe | contact-sheet".into()),
    }
}

fn probe(input: &str) -> Result<(), String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate,avg_frame_rate,nb_frames,duration",
            "-of",
            "json",
            input,
        ])
        .output()
        .map_err(|e| format!("failed to launch ffprobe: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    let parsed: ProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("invalid ffprobe JSON: {e}"))?;
    let stream = parsed.streams.first().ok_or("video stream not found")?;
    println!(
        "{}x{}",
        stream.width.unwrap_or(0),
        stream.height.unwrap_or(0)
    );
    println!(
        "r_frame_rate={}",
        stream.r_frame_rate.as_deref().unwrap_or("?")
    );
    println!(
        "avg_frame_rate={}",
        stream.avg_frame_rate.as_deref().unwrap_or("?")
    );
    println!("duration={}", stream.duration.as_deref().unwrap_or("?"));
    println!("frames={}", stream.nb_frames.as_deref().unwrap_or("?"));
    Ok(())
}

fn contact_sheet(input: &str, output: &str, interval: &str) -> Result<(), String> {
    let seconds: f64 = interval
        .parse()
        .map_err(|_| "seconds-per-frame must be numeric")?;
    if seconds <= 0.0 {
        return Err("seconds-per-frame must be > 0".into());
    }

    if let Some(parent) = Path::new(output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create output directory: {e}"))?;
        }
    }

    let filter = format!("fps=1/{seconds},scale=384:-1,tile=5x11:padding=4:margin=4");
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            input,
            "-vf",
            &filter,
            "-frames:v",
            "1",
            "-y",
            output,
        ])
        .status()
        .map_err(|e| format!("failed to launch ffmpeg: {e}"))?;

    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    println!("wrote {output}");
    Ok(())
}
