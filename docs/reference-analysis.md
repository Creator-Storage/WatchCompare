# Evolution Of Language — reference analysis

This file records measurements taken from the user-supplied reference video. Numbers are separated into **verified** measurements and **provisional** fits so the project never confuses “close” with “locked.”

## Source metadata — verified

- Resolution: **1920 × 1080**
- Frame rate: **60/1 = 60 FPS**
- Duration reported by the video stream: **204.450000 s**
- Frame count: **12,267**
- Video time base: **1/15360 s**
- PTS increment per source frame: **256 ticks**
- Exact source-frame duration: **256/15360 s = 1/60 s = 50/3 ms = 16.666666… ms**
- Last source-frame PTS: **3,140,096 ticks = 204,433.333333… ms**
- Stream end after the last frame duration: **204,450 ms**

Metadata and packet timestamps were read with `ffprobe` from the supplied MP4.

## Temporal rule — verified

**Contact sheets are navigation aids only. They are not animation evidence.**

The reference-lock clock is the source PTS. Every one of the **12,267 source frames** is indexed at its exact packet timestamp. For this particular 60 FPS file there cannot be a new visual source image at every 1 ms; the real source images occur every **50/3 ms**. WatchCompare therefore keeps animation time in integer source ticks internally and exposes milliseconds only as a derived value.

Regenerate the complete timestamp map locally:

```bash
cargo run -p watchcompare-reference -- frame-index reference.mp4 analysis-output/frame-index.csv
```

This produces one row per real source frame instead of inventing interpolated “millisecond frames.”

## Whole-video contact-sheet pass — verified navigation pass

A complete overview was generated locally at one sample every 4 seconds. Dense navigation passes were also generated for the first 12 seconds, the first 6 seconds at 10 samples/second, and the final ~14.45 seconds. The sheets are deliberately not committed.

They confirm four distinct behaviors that must not be collapsed into one generic slide animation:

1. staged intro/reveal while credits occupy the unused right side;
2. continuous horizontal card-train motion;
3. badge-specific motion/shine independent of card translation;
4. a dedicated outro/end-screen composition where the final card remains visible at the right while the end-screen is built on the left, followed by a fade.

## Full every-frame motion pass — verified measurement pass 2

After the navigation pass, the video was decoded **frame by frame for all 12,267 frames**. A 1920×96 strip from the title-panel area was analyzed on every frame (downsampled to 960×48 only for the correlation calculation) and joined to the exact PTS map.

The tracker uses block-anchored phase correlation so it measures displacement from actual neighboring source images rather than from sparse contact-sheet samples. Compression, antialiasing and changing card contents still introduce subpixel measurement noise, so the visual displacement signal is a measurement fixture, not yet the final renderer curve.

Independent one-second steady-state fits from the all-frame pass:

| Window | fitted speed |
|---|---:|
| 20.000–21.000 s | −134.184 px/s |
| 50.000–51.000 s | −133.169 px/s |
| 100.000–101.000 s | −133.739 px/s |

These are consistent with the earlier steady-state estimate around −133 px/s while exposing the actual frame-to-frame rasterization pattern instead of assuming every frame moves by one identical integer amount.

The full 12,267-row visual displacement series is treated as a generated local analysis artifact; only verified measurements are promoted into renderer code/docs.

## First card reveal — verified raster-edge measurement

The first card does **not** simply pop in on one sampled frame. Its title-panel reveal edge was measured on every real source frame at row `y=924`.

- frames **0–4** (`0–66.667 ms`): no card panel is visible at that row;
- frame **5** (`83.333 ms`): first visible edge, **9 px** wide;
- frame **12** (`200 ms`): **60 px**;
- frame **18** (`300 ms`): **183 px**;
- frame **30** (`500 ms`): **348 px**;
- frame **48** (`800 ms`): **437 px**;
- frame **60** (`1000 ms`): **464 px**;
- frame **73** (`1216.667 ms`): **477 px**;
- frame **81** (`1350 ms`): first measured **480 px** full-width state at this row.

The complete per-frame series was generated locally and will be used to fit the actual reveal easing rather than eyeballing a generic ease-in/ease-out preset.

## Intro-to-cruise pan — verified separator tracking

A vertical card separator in the title panel was tracked on **every source frame from 8.0 s onward**. This exposed a detail that sparse sampling completely hid: the initial horizontal pan deliberately accelerates past the eventual cruise speed and then settles back.

- separator remains at **x=960.0** through frame **523**;
- first measured movement: frame **524 = 8733.333 ms**, separator at **x=959.5**;
- acceleration continues through the 9-second range;
- strongest 15-frame local fitted velocity occurs around frame **598 = 9966.667 ms**, approximately **−4.016 px/frame = −240.96 px/s**;
- by around frame **630 = 10500 ms**, the local fitted velocity has returned to approximately **−2.204 px/frame = −132.21 px/s**, close to normal cruise;
- later 15-frame windows remain around roughly **−2.20 to −2.23 px/frame**.

So the old provisional assumption “intro until 12 s, then instantly linear cruise” was wrong. The renderer now keeps frames `< 630` in the Intro stage until the measured non-linear pan curve is installed.

The tracked source-frame series is kept as a local generated measurement artifact until its fitted curve and validation tolerance are locked.

## Card geometry — verified

Measured on a steady-state 1920×1080 frame at ~50 s:

- repeating vertical separator centers: **x = 618, 1095, 1572**;
- therefore separator pitch: **477 px exactly** between those measured separators;
- the first intro card's visible title-panel extent reaches **480 px**, so “card raster width” and “separator-to-separator pitch” must not be treated as the same number without modeling the overlap/border construction;
- visible vertical separator is approximately **5–7 px** after antialias/compression, so the renderer uses **6 px nominal** until the raw edge profile is fitted;
- artwork/title transition occurs between rows **871 → 872**;
- title/description transition occurs between rows **964 → 965**;
- description/bottom-border transition occurs between rows **1074 → 1075**.

Resulting row ranges:

- artwork: **0–871**;
- title panel: **872–964**;
- description panel: **965–1074**;
- bottom border begins: **1075**.

## Steady-state horizontal motion — verified pass 1

Phase correlation was originally run on the moving artwork region across three independent 0.5 second windows:

| Window | measured x shift over 0.5 s |
|---|---:|
| 20.0 → 20.5 s | −66.777465 px |
| 50.0 → 50.5 s | −66.706839 px |
| 100.0 → 100.5 s | −66.725388 px |

Mean displacement:

- **−66.7365639469 px / 0.5 s**
- **−133.4731278938 px/s**
- **−2.22455213156 px/frame at 60 FPS**

The all-frame pass confirms that this is a good steady-state mean, while also showing that the decoded raster does not present one identical measured delta on every frame because subpixel rendering and compression redistribute edge energy between neighboring pixels.

## Additional intro card reveals — verified exact-frame measurements

The second and third normal card reveals were measured independently at the title-panel row rather than inferred from the first card:

- card 2 first reliable visible panel: **frame 125 = 2083.333 ms**; it reaches >=470 px by frame 180 and ~480 px around frame 190;
- card 3 first reliable visible panel: **frame 244 = 4066.667 ms**; it reaches >=470 px by frame 299 and ~480 px around frame 310;
- card 4 is deliberately **not** treated as a normal reveal because the credits overlay is retracting over an already-present card.

The complete source-frame width series is generated locally from the reference; the verified first-visible event frames are promoted into renderer constants. The source-driven generator remains the reproducible source for the larger trace.

## Credits-panel exit — verified exact-frame measurement

A fixed row inside the right-side credits panel was tracked on every source frame through its exit:

- panel left edge is stationary at **x=1431** through frame **395 = 6583.333 ms**;
- at **frame 396 = 6600 ms** it retracts abruptly to **x=1703**, exposing a large portion of card 4 in one source-frame interval;
- then its left edge continues rightward every frame (`1717, 1730, 1741, 1752, ...`);
- frame **428 = 7133.333 ms** leaves only ~27 px;
- frame **429 = 7150 ms** is the first measured frame with the panel fully gone.

That large frame-395→396 jump is present in the decoded source and is therefore retained; it must not be smoothed away into a generic UI easing curve. The full series is in `docs/measurements/credits-exit-exact.csv`.

## Badge polygon — verified raster geometry

A stable first badge was isolated on **frame 220** with no active shine. The connected red component has raster extent **298 × 344 px**. Its clean six-vertex outline is approximated at source-pixel coordinates relative to that component as:

```text
(148,   0)
(  2,  84)
(  0, 255)
(151, 343)
(297, 257)
(297,  84)
```

In global frame-220 coordinates the same contour is approximately `(242,32), (96,116), (94,287), (245,375), (391,289), (391,116)`. These dimensions/vertices are now renderer constants rather than an eyeballed hexagon.

## Badge entrance, overscale and blur — verified frame series; curve fit pending

The badge is independent of card clipping and has a long scale/position/blur settle:

- first actual red badge pixels appear at **frame 34 = 566.667 ms**;
- first badge is only `16×96` visible pixels on frame 34 because it is still entering from beyond the top-left;
- its red width reaches the 298 px canonical width by roughly frame 72, then **overscales** instead of stopping;
- first badge reaches about **332 px red width (~1.114× canonical)** around frames 115–120 before settling;
- the second badge provides a cleaner non-overlapped settle series: around frame 180 its red component is ~374 px wide (**1.255× canonical**) and mostly off the top; by frame 220 it is ~350 px (**1.174×**); by frame 260 ~320 px (**1.074×**); and by frame 292 it reaches the canonical ~298 px scale.

The text does not merely fade in. A canonicalized edge/sharpness measurement on the second badge rises from extremely blurred at frame 168 to sharply resolved by the low-200s. Therefore WatchCompare must model **motion blur/softness and scale simultaneously**, not substitute alpha animation.

The complete first-badge consecutive-frame track is committed as `docs/measurements/badge1-lifecycle-exact.csv`. The longer second-badge sharpness/settle trace is generated locally from the reference and used for the numeric fit without committing source-derived raster artifacts.

The source-frame series is verified; the exact mathematical easing/spline fit is still provisional until renderer image-diff validation.

## Diagonal badge shine — timing verified, geometric fit provisional

The second badge was used because its shine is isolated clearly enough for consecutive-frame tracking:

- faint precursor begins around **frame 232 = 3866.667 ms**;
- strong shine interval: **frames 234–249 = 3900–4150 ms**;
- fading tail remains around frames 250–251;
- shine is visually gone by **frame 252 = 4200 ms**.

The fitted bright band has a major-axis direction near **121.5° from +x** (equivalently a line direction of about −58.5°), and its fitted normal-center advances about **18.45 px per source frame (~1107 px/s)**. The central 80% bright-band width is typically around **32–41 px** in the strongest frames. Because compression and the clipping polygon perturb the fit, angle/speed/width remain **provisional geometry**, while the source-frame visibility window is promoted as verified timing.

The complete fit samples are committed as `docs/measurements/badge-shine-exact.csv`.

## Badge typography — strong evidence, still provisional raster match

Against the supplied authorized Pin Sans files, mask-overlap tests on the stable frame-220 badge currently favor:

- large top line (`7M`): **Pin Sans Heavy around 91 px**;
- lower line (`YEARS AGO`): **Pin Sans Bold around 46–47 px**.

The weight evidence is strong, but exact point size, baseline, hinting and rasterizer behavior remain provisional until rendered glyph masks are diffed against the source. Font binaries remain local and are never committed.

## Badge scale ladder during the intro — verified structural behavior

At frame **523 = 8716.667 ms**, immediately before the card train begins to pan, visible badge sizes differ by card age/position rather than sharing one static scale:

- newest/right badge: ~`302×346`;
- next: ~`274×314`;
- next: ~`246×286`;
- oldest/left: ~`248×283`.

This confirms that older badges continue shrinking as later cards arrive. A renderer that gives every visible badge one final fixed size will diverge from the source even after the initial badge entrance has finished.

## Outro/end screen — structure confirmed; fade timing verified

The final segment transitions from the moving card train into a black end-screen layout. The final `2200 AD` card remains on the right while the left side fills with recommendation blocks, subscription/engagement UI, and a “Video Made By” credit block. This is a dedicated composition, not a normal card exit.

The end-screen fade has now been measured on every real source frame using a stable interior region of the red recommendation panel:

- full measured red level remains unchanged through **frame 12179 = 202983.333 ms**;
- **frame 12180 = 203000 ms** is the first fading source image;
- frame 12181 is already ~97.7% of the baseline red-region brightness;
- ~75% is crossed at frame 12205 = 203416.667 ms;
- ~50% is crossed at frame 12225 = 203750 ms;
- ~25% is crossed at frame 12246 = 204100 ms;
- the tracked region is fully black from **frame 12258 = 204300 ms** through the last frame.

The complete consecutive-frame fade trace is generated locally; its verified boundary frames are promoted into renderer constants. The earlier geometry/build-in portion of the outro still needs the same exact object-tracking treatment before it can be considered locked.

## Acceptance strategy

Reference lock will be considered complete only when:

1. every animation curve is derived from consecutive source frames / exact PTS, never from coarse contact-sheet intervals;
2. the Rust sampler contains measured intro, cruise, badge and outro curves;
3. a software/GPU renderer consumes exactly the same sampled state on Windows and Android;
4. exported reference frames can be compared against selected source frames with automated image diffs;
5. temporal errors are tested on the integer source tick clock;
6. text metrics use the authorized local Pin Sans weights rather than a platform fallback.
