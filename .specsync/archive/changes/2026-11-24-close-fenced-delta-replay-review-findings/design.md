---
change: close-fenced-delta-replay-review-findings
artifact: design
---

# Design

Make the requirement reader's fence recognizer crate-visible and document that internal interface in the validator module spec. The helper's body and reader call sites retain their behavior.

The canonical block matcher scans the original Markdown, tracking byte offsets and applying the helper before matching a heading or deciding the next block boundary. That single scan serves both requirement and spec section targets. The general parser blanker keeps its own existing callers.
