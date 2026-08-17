# Evolution Of Language — reference analysis

This file records measurements taken from the user-supplied reference video. Numbers are separated into **verified** measurements and **provisional** stage boundaries so the project never confuses “close” with “locked.”

## Source metadata — verified

- Resolution: **1920 × 1080**
- Frame rate: **60/1 = 60 FPS**
- Duration reported by the video stream: **204.450000 s**
- Frame count: **12,267**

Metadata was read with `ffprobe` from the supplied MP4.

## Whole-video contact-sheet pass — verified

A complete overview was generated locally at one sample every 4 seconds. Dense passes were also generated for the first 12 seconds, the first 6 seconds at 10 samples/second, and the final ~14.45 seconds. The contact sheets are deliberately not committed; `watchcompare-reference contact-sheet` regenerates them from an authorized local reference file.

The full pass confirms four distinct behaviors that must not be collapsed into one generic slide animation:

1. staged intro/reveal while credits occupy the unused right side;
2. continuous horizontal card-train motion;
3. badge-specific motion/shine independent of card translation;
4. a dedicated outro/end-screen composition where the final card remains visible at the right while the end-screen is built on the left, followed by a fade.

## Card geometry — verified

Measured on a steady-state 1920×1080 frame at ~50 s:

- repeating vertical separator centers: **x = 618, 1095, 1572**;
- therefore card pitch: **477 px exactly** between the measured separators;
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

Phase correlation was run on the moving artwork region across three independent 0.5 second windows:

| Window | measured x shift over 0.5 s |
|---|---:|
| 20.0 → 20.5 s | −66.777465 px |
| 50.0 → 50.5 s | −66.706839 px |
| 100.0 → 100.5 s | −66.725388 px |

Mean displacement:

- **−66.7365639469 px / 0.5 s**
- **−133.4731278938 px/s**
- **−2.22455213156 px/frame at 60 FPS**

A 477 px card pitch therefore advances in about **3.573753066 s** during the steady state.

The small spread in measured shifts is consistent with video compression/subpixel estimation. A later pass will fit the source's exact discrete frame positions rather than assuming the continuous mean is the final answer.

## Intro — confirmed structure, curves still provisional

Dense contact sheets show that the intro is not simply the steady card train starting off-screen. Cards are staged into the composition while the credits block occupies the unused right region. Oversized/cropped badge states and badge settling are visible during incoming cards. Exact keyframe times, easing curves, badge scale/position, blur, and shine timing are still being fitted frame-by-frame.

The Rust model currently isolates `Intro` as its own stage so these curves can replace the provisional sampler without changing editor code.

## Badge motion — confirmed structure, numeric fit pending

The badge is not baked into a single static card image. The reference shows independent badge behavior including entrance/settling and a diagonal highlight/shine. The lock pass must recover at least:

- badge base polygon geometry;
- anchor point and final x/y;
- initial x/y/scale for each reveal;
- easing and overshoot/settle values;
- blur/softness during fast motion;
- diagonal shine angle, width, opacity, and start/end times;
- text baseline/leading inside the badge.

## Outro — confirmed structure, curves still provisional

The final segment transitions from the moving card train into a black end-screen layout. The final `2200 AD` card remains on the right while the left side fills with recommendation blocks, subscription/engagement UI, and a “Video Made By” credit block. The complete frame then fades down.

This must be a dedicated timeline stage rather than a normal card exit.

## Acceptance strategy

Reference lock will be considered complete only when:

1. the Rust sampler contains measured intro, cruise, badge and outro curves;
2. a software/GPU renderer consumes exactly the same sampled state on Windows and Android;
3. exported reference frames can be compared against selected source frames with automated image diffs;
4. timing errors are tested at the source frame rate, not judged only by eye;
5. text metrics use the authorized local Pin Sans weights rather than a platform fallback.
