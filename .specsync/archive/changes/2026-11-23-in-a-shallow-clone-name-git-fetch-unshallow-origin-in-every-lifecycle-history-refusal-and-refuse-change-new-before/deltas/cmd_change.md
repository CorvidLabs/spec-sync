## ADDED

### REQUIREMENT REQ-cmd-change-044

Text `change status` in a shallow clone SHALL give REQ-change-173's refusal instead of the correction-ledger text.

Acceptance Criteria
- In a shallow clone deep enough for the project's workflow-v2 baseline but not for a change's recorded base commit, text `change status <id>` says the clone is shallow and names `git fetch --unshallow origin`, not "correction ledger integrity is invalid".
- In a full clone, a corrupt correction ledger keeps its own text.
