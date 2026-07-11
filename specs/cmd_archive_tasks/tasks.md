---
spec: cmd_archive_tasks.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add an integration test that drives `archive-tasks` end-to-end through the CLI (the delegate `archive` module is unit-tested, but the wrapper's output/`--dry-run` formatting is not)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified wrapper delegates to `archive::archive_tasks` and matches the empty-result / dry-run / write paths
- [x] Confirmed delegate logic is covered by `archive` inline tests (`test_archive_completed_tasks`, `test_archive_no_completed`, `test_archive_preserves_existing`)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
