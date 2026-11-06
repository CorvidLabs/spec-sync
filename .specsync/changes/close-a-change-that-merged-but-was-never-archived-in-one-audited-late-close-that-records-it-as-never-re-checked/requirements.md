---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: requirements
---

# Requirements

REQ-change-143 (ADDED, deltas/change.md) carries the whole behavior. The cmd_change and cli_args specs
take direct edits inside the declared paths: the `close-late` handler and argument rows in their Public
API tables, and their Change Log rows. No existing requirement is modified or retired.

Traceability:

| HI | REQ-change-143 criterion |
| --- | --- |
| SHIP-12 | eligibility; the archive preflight no longer stops at a closed-late change |
| SHIP-12.a | `late-close.json` fields; status, ship-status and JSON report the late close and `verified: false` |
| SHIP-12.b | not authenticated, never successor coverage, refused by reopen, review and finalize; audit counts it closed |
| SHIP-12.c | no delivery-input read, no manifest or digest rebuild, no verification |
| SHIP-12.d | `--actor` and `--reason` required and recorded |
| SHIP-12.e | the package must be on the remote default branch; otherwise refused |
| SHIP-12.f | several IDs, all checked first; one refusal writes nothing; `--dry-run` |
