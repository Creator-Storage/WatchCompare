# Outro build — exact source-frame track

All timing below comes from consecutive decoded source frames on the exact 1/15360 source clock. No contact-sheet interval is used as animation evidence.

## Final card-train stop

The normal card train continues through frame 11842 and reaches its final measured position at **frame 11843 = 197383.333 ms**. The final 2200 AD card then remains frozen on the right.

- frame 11842: measured card-train x = **-25221.0 px**
- frame 11843: measured card-train x = **-25221.0 px**
- later outro frames retain the same x

The full frame-630→11843 raster-motion trace is encoded in `reference_card_train_delta_half_px.txt`; each character preserves one measured half-pixel frame delta.

## Black wipe

The left 1440 px is covered top-to-bottom while the final card remains visible. Exact measured wipe-bottom y for frames 11868..11884:

```text
23, 67, 125, 191, 267, 351, 439, 531, 621, 709, 792, 871, 939, 996, 1039, 1067, 1079
```

- first wipe frame: **11868 = 197800 ms**
- full-height wipe: **11884 = 198066.667 ms**

## Recommendation / Video Made By group

The recommendation panels and the gray credit block enter as one coordinated vertical group from frame 11901 through 11911.

Recommendation-panel top y:

```text
-60, -14, 27, 66, 102, 132, 159, 180, 196, 206, 210
```

Gray credit-block top y:

```text
470, 515, 557, 596, 632, 663, 690, 711, 727, 737, 740
```

- group first visible: **11901 = 198350 ms**
- group settled: **11911 = 198516.667 ms**

## CTA bar

The outer white CTA bar first appears on **frame 11913 = 198550 ms** and grows in real stepped source-frame increments rather than one smooth assumed scale tween. It reaches the measured stable outer bbox by **frame 11957 = 199283.333 ms**.

First bbox:

```text
x=696 y=93 width=82 height=18
```

Settled bbox:

```text
x=467 y=36 width=541 height=131
```

The complete 45-frame bbox series is compiled into `watchcompare-render` as `OUTRO_CTA_BBOX`.

## Fade

- last full-brightness frame: **12179 = 202983.333 ms**
- first faded frame: **12180 = 203000 ms**
- tracked end-screen red region fully black: **12258 = 204300 ms**

The measured 80-frame brightness series is compiled into the renderer. It deliberately retains real source plateaus and raster/compression steps rather than replacing them with a guessed linear alpha ramp.

## Still pending

The major outro geometry/timing is now source-frame locked. Remaining outro fidelity work is the CTA's internal icon/text sequence and compositor/image-diff validation of edges, antialiasing, text and color.