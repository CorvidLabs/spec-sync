## ADDED

### REQUIREMENT REQ-change-151

Failed verification command evidence SHALL carry a bounded `output` field with the findings that failed it, so a recorded failure is classifiable without re-running the command.

Acceptance Criteria
- `CommandEvidence` gains `output: Option<String>`, serde-defaulted and omitted when `None`; passing records serialize byte-identically to before.
- A failed scoped spec↔code check records its findings text (bounded to the last 4 KiB) in `output`.
- A gate-refusal evidence entry records the refusal texts the same way.
- Records written by an older binary parse without error.
