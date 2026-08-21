# Plan

1. `ArchiveIntroduction` carries committed `approvals.json` bytes, not a count.
2. `ledger_succession` — prefix-identical growth plus a matching `superseded_approval`.
3. `admissible_archive_introductions` supersedes on `NotASuccessor`; delete
   `approval_ledger_generation`.
4. Reopen the working-tree fallback for the writing process only, gated on that succession.
5. Widen the scoped-review path set to every archive directory the package has occupied, so a
   reopen's own move is not read as a deletion.
6. Verify against: the three vectors with and without a forged generation, drills 013/049/069,
   the full board, the full suite, and the corpus per risk class.
