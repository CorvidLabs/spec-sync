---
change: a-failed-verification-records-the-findings-that-failed-it
artifact: design
---

# Design

`CommandEvidence.output: Option<String>` is serde-defaulted and skipped when `None`, so a passing
record keeps its bytes and an older record still parses. `bounded_failure_output(findings)` builds it
from the findings that failed the step: errors, plus warnings only when `--strict` made them
failures, and a gate refusal's texts. Over 4 KiB it keeps the tail: the cut moves forward to a
character boundary, then to a line boundary when one falls near the cut and text follows it, after a
leading `…` line. Empty finding text records no output.
