# Exact-frame fixture crate status

`watchcompare-fixtures` is now a workspace member and is exercised before the renderer in CI.

Current exact-source fixtures include:

- second badge visible-core growth, source frames 150–169;
- second badge fitted scale/x/y transform, source frames 170–300;
- second badge text reveal/sharpening area metric, source frames 186–265;
- CTA internal build event boundaries (subscribe, like, bell, underline, dislike);
- like-button blue activation level, source frames 12053–12078;
- subscribed-button pulse geometry, source frames 12115–12140;
- bell-fill activation metrics, source frames 12169–12180.

The CTA cursor is confirmed to enter at source frame 12007 and is visible moving through Like → Subscribe → Bell, but its complete x/y trajectory is deliberately **not** marked locked until the pointer can be separated from the changing CTA raster on every consecutive frame.

Badge text blur sigma is likewise still pending: the exact source reveal metric is stored, while a simplistic Gaussian-only fit was rejected because the source combines scale/reveal/blur rather than one blur parameter.
