# Typography lock — source-frame measurement

Reference source: `Comparison: Evolution Of Language (400,000 BC - 2026)` at exact source frame 3000 (50,000 ms).

The supplied local Pin Sans MacOS files were raster-tested against decoded source pixels. Font binaries remain local and are not committed.

## Card title

**Verified family/weight/size:** `Pin Sans MacOS Heavy`, **40 px** at the 1920×1080 reference raster.

Independent checks:

- `Punctuation Invented`: source black-ink bbox is ~379×28 px at the compressed-source threshold; Pin Sans Heavy 40 has a 379 px nominal ink width and produced the strongest raster correlation of the supplied weights/sizes.
- `Tamil Is`: source width 128 px; Heavy 40 nominal width 128 px.
- `First Spoken`: source width ~220 px; Heavy 40 nominal width 222 px.

The two-line title source rows occupy approximately y=882–907 and y=922–948 inside the global 872–964 title panel, confirming that line placement must be explicit rather than delegated to platform text layout.

## Card description

**Verified family/weight/size:** `Pin Sans MacOS Heavy`, **28 px** at the 1920×1080 reference raster.

Independent checks:

- `A Greek librarian` / `invents dots for pauses`: source line widths ~214 px / 290 px; Heavy 28 matches the longer line at ~289 px and gives the best combined raster correlation of the supplied candidate weights/sizes.
- `The oldest language` / `still spoken every day`: source line widths ~252 px / 270 px; Heavy 28 nominal widths ~250 px / 267 px.

The description lines begin around global y=980 and y=1009 in these steady-state cards. Measured line separation is therefore about 29 px at the source raster.

## Rendering rule

Windows and Android must not use operating-system font substitution or native WebView text for exported frames. The deterministic compositor must load the authorized local font asset and place glyphs on the reference coordinate system. UI preview text may be scaled for display, but reference/export raster metrics come from the shared renderer.

These values are considered geometry-locked. Anti-aliasing/compositing still requires image-diff calibration in the renderer because the MP4 source includes encoding artifacts that are not font metrics.
