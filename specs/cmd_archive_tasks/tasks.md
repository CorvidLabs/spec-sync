---
spec: cmd_archive_tasks.spec.md
---

## Tasks

- [ ] Add an integration test that drives `archive-tasks` end-to-end through the CLI (the delegate `archive` module is unit-tested, but the wrapper's output/`--dry-run` formatting is not)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified wrapper delegates to `archive::archive_tasks` and matches the empty-result / dry-run / write paths
- [x] Confirmed delegate logic is covered by `archive` inline tests (`test_archive_completed_tasks`, `test_archive_no_completed`, `test_archive_preserves_existing`)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
