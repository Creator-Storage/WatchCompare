# Reference image-diff gate

WatchCompare treats decoded source frames as the acceptance target. The diff gate never rescales either image: reference and candidate must have identical pixel dimensions.

Local usage with authorized reference material:

```bash
cargo run -p watchcompare-diff -- \
  analysis-output/source-f00234.png \
  analysis-output/rendered-f00234.png \
  --threshold 8 \
  --max-mae 2.0 \
  --max-fraction 0.01 \
  --diff analysis-output/diff-f00234.png
```

The CLI reports JSON metrics:

- RGB mean absolute error (MAE)
- RGB root mean square error (RMSE)
- maximum channel error
- fraction of pixels that are exactly equal
- count/fraction of pixels whose maximum RGB channel error exceeds the selected threshold

Exit code `0` means the configured gate passed, `2` means the candidate rendered successfully but missed the fidelity gate, and `1` means the comparison could not be performed.

The threshold values above are only a working calibration point. A component is not marked source-locked merely because it passes a broad whole-frame threshold: localized badge, text, shine and cursor regions should also be tested with stricter crop-specific gates while the compositor is calibrated.

Reference video, decoded source PNGs and supplied font binaries remain local. The repository contains the deterministic comparison code and numeric measurements, not redistributed source assets.
