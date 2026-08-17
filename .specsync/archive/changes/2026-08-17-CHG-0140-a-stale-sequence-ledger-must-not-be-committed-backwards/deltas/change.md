## ADDED

### REQUIREMENT REQ-change-070

A lifecycle commit SHALL NOT record a change sequence ledger below the highest sequence already committed, and SHALL disclose any raise it performs.

Acceptance Criteria
- Before staging, a working-tree ledger lower than the committed high-water mark is raised to it, so no lifecycle commit can lower the recorded mark.
- A working-tree ledger at or above the committed mark is left exactly as the author wrote it, because a newer claim is the ordinary result of allocating a change and must not be overwritten.
- The raise is reported on a stream that survives quiet output and does not contaminate a machine-readable payload, naming both the previous and the adopted value.
- Acknowledged collisions recorded on either side are preserved across the raise rather than replaced by one side's copy.
- Every staging site in the lifecycle applies the rule, so a commit path added later cannot reintroduce the regression by bypassing one of them.

### REQUIREMENT REQ-change-071

Validating change sequences SHALL refuse a ledger below the high-water mark the default branch has already published, whether or not the higher-numbered workspaces are present on disk.

Acceptance Criteria
- A local ledger below the default branch's recorded sequence is refused even when no higher-numbered workspace directory exists locally, which is the ordinary state of a fresh clone or an unfetched branch.
- The refusal names both the claimed and the published sequence, and states the command that restores the ledger.
- A local ledger at or above the published mark is accepted, so ordinary allocation is unaffected.
- The published mark is read from the same source the allocation floor already consults, rather than from a second implementation of the same lookup.
