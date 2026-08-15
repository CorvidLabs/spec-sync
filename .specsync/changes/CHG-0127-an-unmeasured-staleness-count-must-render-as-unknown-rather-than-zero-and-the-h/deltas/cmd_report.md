## ADDED

### REQUIREMENT REQ-cmd-report-004

An unmeasured staleness count SHALL render as unknown, never as zero.

Acceptance Criteria
- Text says the count is unknown; JSON emits `null`.
- A number appears only when at least one module's staleness was actually measured.
- A tree with real git history reports its count exactly as before, so the count is made honest rather than removed.
