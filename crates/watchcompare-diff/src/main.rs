use std::{env, path::PathBuf, process::ExitCode};
use watchcompare_diff::{compare_images, difference_image, passes, DiffLimits};

fn usage() -> &'static str {
    "usage: watchcompare-diff <reference.png> <candidate.png> [--threshold N] [--max-mae X] [--max-fraction X] [--diff out.png]"
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(error) => {
            eprintln!("watchcompare-diff: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool, String> {
    let mut args = env::args().skip(1);
    let reference_path = PathBuf::from(args.next().ok_or_else(|| usage().to_owned())?);
    let candidate_path = PathBuf::from(args.next().ok_or_else(|| usage().to_owned())?);

    let mut limits = DiffLimits::default();
    let mut diff_path: Option<PathBuf> = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--threshold" => {
                limits.threshold = args
                    .next()
                    .ok_or("--threshold requires a value")?
                    .parse()
                    .map_err(|_| "--threshold must be an integer from 0 to 255")?;
            }
            "--max-mae" => {
                limits.max_mean_abs_error = args
                    .next()
                    .ok_or("--max-mae requires a value")?
                    .parse()
                    .map_err(|_| "--max-mae must be numeric")?;
            }
            "--max-fraction" => {
                limits.max_fraction_above_threshold = args
                    .next()
                    .ok_or("--max-fraction requires a value")?
                    .parse()
                    .map_err(|_| "--max-fraction must be numeric")?;
            }
            "--diff" => {
                diff_path = Some(PathBuf::from(
                    args.next().ok_or("--diff requires an output PNG path")?,
                ));
            }
            _ => return Err(format!("unknown option {flag}\n{}", usage())),
        }
    }

    if !(0.0..=1.0).contains(&limits.max_fraction_above_threshold) {
        return Err("--max-fraction must be between 0 and 1".into());
    }
    if limits.max_mean_abs_error < 0.0 {
        return Err("--max-mae must be >= 0".into());
    }

    let reference = image::open(&reference_path)
        .map_err(|e| format!("open {}: {e}", reference_path.display()))?;
    let candidate = image::open(&candidate_path)
        .map_err(|e| format!("open {}: {e}", candidate_path.display()))?;

    let metrics = compare_images(&reference, &candidate, limits.threshold)
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&metrics).map_err(|e| e.to_string())?
    );

    if let Some(path) = diff_path {
        let diff = difference_image(&reference, &candidate).map_err(|e| e.to_string())?;
        diff.save(&path)
            .map_err(|e| format!("save {}: {e}", path.display()))?;
        eprintln!("wrote {}", path.display());
    }

    let accepted = passes(metrics, limits);
    if !accepted {
        eprintln!(
            "reference lock failed: MAE {:.6} (limit {:.6}), fraction>{} = {:.8} (limit {:.8})",
            metrics.mean_abs_error,
            limits.max_mean_abs_error,
            limits.threshold,
            metrics.fraction_above_threshold,
            limits.max_fraction_above_threshold
        );
    }
    Ok(accepted)
}
