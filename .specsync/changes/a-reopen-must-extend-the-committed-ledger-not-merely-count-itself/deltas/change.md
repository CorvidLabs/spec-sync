## ADDED

### REQUIREMENT REQ-change-088

A later generation of a change's terminal evidence SHALL be trusted only when it extends the generation already committed, and closing evidence that history has not seen SHALL be presentable only by the process writing that package out of the active workspace.

Acceptance Criteria
- A generation is accepted as later only when it contains, unrewritten, every approval and reopen event the committed generation already holds, because a count of reopen events is written by whoever writes the file and so distinguishes nothing.
- Rewriting any earlier entry while appending a new one is refused, so a forged reopen cannot launder a tampered approval by appearing to advance the ledger.
- A change that has been genuinely reopened can be closed again, because the evidence for a new generation necessarily does not yet exist in history at the moment it is being written.
- Evidence that history has not seen is accepted only from the process writing the package out of the active workspace, so a working tree cannot speak for a package that history already holds.
