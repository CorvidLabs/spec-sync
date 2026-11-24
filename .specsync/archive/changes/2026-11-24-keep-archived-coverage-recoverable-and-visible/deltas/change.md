## ADDED

### REQUIREMENT REQ-change-reopened-coverage-receipt-001

When a stale Archived change is successfully reopened, SpecSync SHALL retire its valid generated `coverage.json` for that archive generation in the same durable file transaction as the reopened state and ledger. A failed file publication SHALL restore the prior receipt bytes with the other original files, and a refused reopen SHALL preserve the original archive. Re-finalization SHALL compute coverage for the new generation. A file left by the user in an Accepted or active workspace, an invalid receipt, or bytes changed during publication SHALL NOT be treated as an eligible generated receipt. A genuinely absent file SHALL be distinguished from a present unreadable or invalid receipt. Neither its contents nor a diagnostic about it SHALL determine authenticated terminal archive evidence.

Acceptance Criteria
- A committed, provisionally covered archive can reopen, re-check, and finalize again. A new generation without temporary coverage has no receipt, and its previous committed archive retains its original bytes.
- A write failure during an archived reopen restores the original receipt and package. An unexpected active coverage file is preserved and remains an archival conflict.
- A present receipt with invalid schema, change identity, entries, or unreadable contents is surfaced without suppressing independently authenticated archive evidence.
