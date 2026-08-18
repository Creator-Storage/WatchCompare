# CI full verification

This branch exists to force a pull-request GitHub Actions run so every Rust crate can be observed through the connected GitHub Actions tooling.

The verification target is the full current Rust core workflow, including the exact-frame fixtures, cursor trajectory, mid-video CTA fixture, deterministic renderer, exact scene sampler, deterministic compositor, strict pixel-diff core, and reference CLI check.
