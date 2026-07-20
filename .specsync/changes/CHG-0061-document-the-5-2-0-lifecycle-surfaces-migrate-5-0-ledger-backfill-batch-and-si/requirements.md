---
change: CHG-0061-document-the-5-2-0-lifecycle-surfaces-migrate-5-0-ledger-backfill-batch-and-si
artifact: requirements
---

# Requirements

The 5.2.0 user-facing lifecycle surfaces SHALL be documented where operators look for them.

- `cli.md` covers `migrate 5.0`, both `correct-owner` forms, and `change supersede` accurately.
- `workflow.md` explains squash-merged archival trust and the staleness/migrate diagnostics.
- `cross-project-refs.md` states the inert registry stub tolerance and its fail-closed boundary.
- `AGENTS.md` quick reference lists the SDD lifecycle commands and `migrate 5.0`.
- No code, canonical spec content, or command behavior changes.
