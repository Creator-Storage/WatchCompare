# Exact-frame measurement fixtures

These fixtures come from consecutive decoded source frames on the reference's exact 1/15360 time base. Contact sheets are not timing evidence.

Committed exact traces:

- `credits-exit-exact.csv` — credits overlay edge, frames 340–440.
- `badge1-lifecycle-exact.csv` — first badge red-component position/extent, frames 34–133.
- `badge-shine-exact.csv` — second-badge shine-band measurements, frames 226–254.

Larger traces (full-video phase correlation, second-badge blur/settle, multi-card reveal widths and the outro fade intensity curve) are generated locally from the authorized source video. Their verified event boundaries are promoted into `watchcompare-render` constants so the repository does not need to contain source raster frames or the reference MP4.

The next lock step is fitting these source-frame traces into deterministic renderer curves and validating rendered source-frame timestamps with image diffs.
