---
change: bounded-failure-output-never-splits-a-multi-byte-character-and-gate-refusal-evidence-is-tested
artifact: delta
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-151

Failed verification command evidence SHALL carry a bounded `output` field with the findings that failed it, so a recorded failure is classifiable without re-running the command.

Acceptance Criteria
- `CommandEvidence` gains `output: Option<String>`, serde-defaulted and omitted when `None`; passing records serialize byte-identically to before.
- A failed scoped spec↔code check records its findings text (bounded to the last 4 KiB) in `output`.
- A gate-refusal evidence entry records the refusal texts the same way.
- Records written by an older binary parse without error.
- The 4 KiB bound never splits a multi-byte character: findings over the bound keep a tail that starts on a character boundary, and on a line boundary when one falls inside the tail, after a leading `…` line, so a large non-ASCII failure is recorded instead of panicking.
