---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: tasks
---

# Tasks

- [x] Implement transactional `add_acceptance_owner_corrections` and `--all-missing` discovery.
- [x] Extend Clap `correct-owner` grammar for repeated paths/specs, manifest, and `--all-missing`.
- [x] Wire the change command adapter for batch rendering.
- [x] Add unit and integration tests for batch success, atomic partial failure, and discovery.
- [x] Map REQ-change-038, REQ-cli-args-006, and REQ-cmd-change-004; update module companions.
- [x] Run pre-acceptance formatting, lint, tests, and release validators.
