use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    io::Write,
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
    time_base: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PacketProbeOutput {
    streams: Vec<PacketProbeStream>,
    packets: Vec<Packet>,
}

#[derive(Debug, Deserialize)]
struct PacketProbeStream {
    time_base: String,
}

#[derive(Debug, Deserialize)]
struct Packet {
    pts: Option<i64>,
    duration: Option<i64>,
    flags: Option<String>,
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
        Some("frame-index") => {
            let input = args
                .next()
                .ok_or("usage: watchcompare-reference frame-index <video> <out.csv>")?;
            let output = args
                .next()
                .ok_or("usage: watchcompare-reference frame-index <video> <out.csv>")?;
            frame_index(&input, &output)
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
        _ => Err("commands: probe | frame-index | contact-sheet".into()),
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
            "stream=width,height,r_frame_rate,avg_frame_rate,nb_frames,duration,time_base",
            "-of",
            "json",
            input,
        ])
        .output()
        .map_err(|e| format!("failed to launch ffprobe: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    let parsed: ProbeOutput =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("invalid ffprobe JSON: {e}"))?;
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
    println!("time_base={}", stream.time_base.as_deref().unwrap_or("?"));
    println!("duration={}", stream.duration.as_deref().unwrap_or("?"));
    println!("frames={}", stream.nb_frames.as_deref().unwrap_or("?"));
    Ok(())
}

fn parse_rational(value: &str) -> Result<(i64, i64), String> {
    let (num, den) = value
        .split_once('/')
        .ok_or_else(|| format!("invalid rational: {value}"))?;
    let num = num
        .parse::<i64>()
        .map_err(|_| format!("invalid rational numerator: {value}"))?;
    let den = den
        .parse::<i64>()
        .map_err(|_| format!("invalid rational denominator: {value}"))?;
    if den == 0 {
        return Err(format!("zero rational denominator: {value}"));
    }
    Ok((num, den))
}

/// Writes one row for every encoded video frame packet using the source PTS.
///
/// This is intentionally different from making a denser contact sheet: a 60 FPS
/// source only contains a new source image every 50/3 ms. The exact PTS/tick map
/// is the temporal source of truth for reference-lock work.
fn frame_index(input: &str, output: &str) -> Result<(), String> {
    if let Some(parent) = Path::new(output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("create output directory: {e}"))?;
        }
    }

    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_streams",
            "-show_packets",
            "-show_entries",
            "stream=time_base:packet=pts,duration,flags",
            "-of",
            "json",
            input,
        ])
        .output()
        .map_err(|e| format!("failed to launch ffprobe: {e}"))?;

    if !probe.status.success() {
        return Err(String::from_utf8_lossy(&probe.stderr).into_owned());
    }

    let parsed: PacketProbeOutput = serde_json::from_slice(&probe.stdout)
        .map_err(|e| format!("invalid packet ffprobe JSON: {e}"))?;
    let stream = parsed.streams.first().ok_or("video stream not found")?;
    let (time_base_num, time_base_den) = parse_rational(&stream.time_base)?;

    let mut packets: Vec<_> = parsed
        .packets
        .into_iter()
        .filter(|packet| packet.pts.is_some())
        .collect();
    packets.sort_by_key(|packet| packet.pts.unwrap_or(i64::MAX));

    let mut file = File::create(output).map_err(|e| format!("create {output}: {e}"))?;
    writeln!(
        file,
        "frame,pts_ticks,timestamp_ms,duration_ticks,duration_ms,keyframe"
    )
    .map_err(|e| format!("write {output}: {e}"))?;

    for (frame, packet) in packets.iter().enumerate() {
        let pts = packet.pts.unwrap_or(0);
        let duration = packet.duration.unwrap_or(0);
        let timestamp_ms = pts as f64 * time_base_num as f64 * 1000.0 / time_base_den as f64;
        let duration_ms =
            duration as f64 * time_base_num as f64 * 1000.0 / time_base_den as f64;
        let keyframe = packet
            .flags
            .as_deref()
            .map(|flags| flags.contains('K'))
            .unwrap_or(false);
        writeln!(
            file,
            "{frame},{pts},{timestamp_ms:.6},{duration},{duration_ms:.6},{}",
            if keyframe { 1 } else { 0 }
        )
        .map_err(|e| format!("write {output}: {e}"))?;
    }

    println!(
        "wrote {} frame timestamps to {output}; time_base={}/{}",
        packets.len(),
        time_base_num,
        time_base_den
    );
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
            fs::create_dir_all(parent).map_err(|e| format!("create output directory: {e}"))?;
        }
    }

    // Contact sheets are navigation aids only. They must never be used as the
    // temporal source of truth for frame-accuracy measurements.
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
