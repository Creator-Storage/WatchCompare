# WatchCompare

Creative Network launcher and deterministic comparison-video studio for **Windows and Android**.

WatchCompare is Rust-first. The application shell uses Tauri 2 so Windows and Android share the same project, while animation timing and geometry live in an independent Rust crate. The WebView is the editor/launcher UI; it is not the source of truth for exported-frame motion.

## Reference-lock goal

The supplied **Comparison: Evolution Of Language (400,000 BC - 2026)** video is the first frame-accuracy target. “Looks close” is not the acceptance criterion. Card geometry, continuous scroll, badge entrance/settling/shine, intro staging, title/description panels, and the special end screen are measured from source frames and represented explicitly in the timeline.

Verified baseline facts are in [`docs/reference-analysis.md`](docs/reference-analysis.md).

## Workspace

- `src-tauri/` — Tauri 2 Windows/Android shell and Rust commands
- `src/` — Creative Network launcher/editor frontend
- `crates/watchcompare-render/` — deterministic reference timeline/state sampler
- `crates/watchcompare-reference/` — local reference-video metadata/contact-sheet tooling
- `docs/reference-analysis.md` — measured facts, confidence, and remaining lock work
- `assets/fonts/` — local Pin Sans import manifest; font binaries are not committed

## Build

Install the Tauri CLI:

```bash
cargo install tauri-cli --version "^2.0.0" --locked
```

Desktop development:

```bash
cargo tauri dev
```

Android initialization/build:

```bash
cargo tauri android init
cargo tauri android build --apk
```

Windows release build:

```bash
cargo tauri build
```

## Reference analysis

Generate a local contact sheet without committing the source video or frames:

```bash
cargo run -p watchcompare-reference -- contact-sheet \
  "Comparison： Evolution Of Language (400,000 BC - 2026).mp4" \
  analysis-output/full-reference.jpg 4
```

Inspect metadata:

```bash
cargo run -p watchcompare-reference -- probe reference.mp4
```

## Fidelity policy

Measured constants are tagged as `verified` or `provisional`. Provisional values are never silently promoted to “exact.” The renderer profile will only be marked locked once automated frame comparisons pass the agreed pixel/temporal tolerances.
