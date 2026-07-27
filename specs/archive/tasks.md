---
spec: archive.spec.md
---

## Tasks

## Post-5.0 Test Debt

(none open)

## Done

- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)
- [x] Implement `archive_tasks`, `archive_completed_tasks`, and `count_completed_tasks`
- [x] Unit tests for archive, no-completed, and preserve-existing cases
- [x] Add CLI integration coverage for successful preview and structured failure reporting
- [x] Return typed planned, succeeded, rolled-back, and failed operation collections
- [x] Preflight and stage every operation before atomic publication
- [x] Preserve destination permissions and roll back prior publishes after a late failure
- [x] Cover planning, middle-staging, middle-publish, rollback, permission, and dry-run/apply parity cases

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
