---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: plan
---

# Plan

1. Add REQ-change-038, REQ-cli-args-006, and REQ-cmd-change-004 with matching Public API /
   Invariants / Contract deltas.
2. Implement batch domain API with validate-all-then-write semantics and `--all-missing` discovery.
3. Extend Clap grammar and the change command adapter.
4. Add unit and integration tests for batch success, atomic partial failure, manifest parsing, and
   `--all-missing`.
5. Update module companions (`context.md`, `tasks.md`, `testing.md`) before verify/accept.
6. Run verify gates and accept.
