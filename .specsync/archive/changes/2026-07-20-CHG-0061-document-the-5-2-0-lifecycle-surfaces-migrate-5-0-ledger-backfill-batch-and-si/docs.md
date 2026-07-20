---
change: CHG-0061-document-the-5-2-0-lifecycle-surfaces-migrate-5-0-ledger-backfill-batch-and-si
artifact: docs
---

# Docs

- `cli.md`: document `specsync migrate 5.0` (deterministic digest backfill, idempotency,
  `--dry-run`, per-change failure isolation) and both `correct-owner` forms (single
  `--path`/`--spec`, and batch with repeated flags, `--manifest`, and `--all-missing`), plus
  `change supersede` in the lifecycle command list.
- `workflow.md`: describe squash-merged archival trust (recording anchors) and the actionable
  staleness/migrate diagnostics operators see.
- `cross-project-refs.md`: document that inert 5.0.1 registry stubs load as absent while
  invalid non-inert registries still fail closed.
- `AGENTS.md`: refresh the quick reference with the SDD lifecycle and migration commands.
