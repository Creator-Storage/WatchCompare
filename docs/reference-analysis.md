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

## Badge motion — structure confirmed, exact numeric fit still pending

The badge is not baked into a single static card image. The reference shows independent badge behavior including entrance/settling and a diagonal highlight/shine. The exact-frame pass is now the required method for recovering:

- badge base polygon geometry;
- anchor point and final x/y;
- initial x/y/scale for each reveal;
- easing and overshoot/settle values;
- blur/softness during fast motion;
- diagonal shine angle, width, opacity, and start/end source-frame PTS;
- text baseline/leading inside the badge.

The intro's red badge already shows visible overscale/settle across consecutive 16.667 ms frames; it will be tracked independently from card clipping so those two animations are not conflated.

## Outro — confirmed structure, curves still provisional

The final segment transitions from the moving card train into a black end-screen layout. The final `2200 AD` card remains on the right while the left side fills with recommendation blocks, subscription/engagement UI, and a “Video Made By” credit block. The complete frame then fades down.

This must be a dedicated timeline stage rather than a normal card exit. It will receive the same PTS-exact treatment as the intro.

## Acceptance strategy

Reference lock will be considered complete only when:

1. every animation curve is derived from consecutive source frames / exact PTS, never from coarse contact-sheet intervals;
2. the Rust sampler contains measured intro, cruise, badge and outro curves;
3. a software/GPU renderer consumes exactly the same sampled state on Windows and Android;
4. exported reference frames can be compared against selected source frames with automated image diffs;
5. temporal errors are tested on the integer source tick clock;
6. text metrics use the authorized local Pin Sans weights rather than a platform fallback.
