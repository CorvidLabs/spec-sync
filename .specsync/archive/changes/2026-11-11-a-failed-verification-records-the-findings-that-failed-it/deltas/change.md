---
change: a-failed-verification-records-the-findings-that-failed-it
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-153

Failed verification command evidence SHALL carry a bounded `output` field with the findings that failed it, so a recorded failure is classifiable without re-running the command.

Acceptance Criteria
- `CommandEvidence` gains `output: Option<String>`, serde-defaulted and omitted when `None`; passing records serialize byte-identically to before.
- A failed scoped spec↔code check records in `output` the findings that failed it, bounded to the last 4 KiB: its errors, and its warnings only when `--strict` made them failures, so an error followed by many warnings is still recorded.
- A gate-refusal evidence entry records the refusal texts the same way.
- Records written by an older binary parse without error.
- The 4 KiB bound never splits a multi-byte character: findings over the bound keep a tail that starts on a character boundary, and on a line boundary when one falls inside the tail, after a leading `…` line, so a large non-ASCII failure is recorded instead of panicking.
- A cut tail starts on a line boundary only when one falls near the cut and text follows it; output with no finding text is not recorded.
