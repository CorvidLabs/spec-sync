---
change: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
artifact: plan
---

# Plan

1. Add `--commit` / `--push` flags on `ChangeAction::Check` in `src/cli.rs`.
2. Implement `run_checked_commit` in `src/commands/change.rs`.
3. Document in `cli` and `cmd_change` specs via deltas.
4. Cover with CLI parse tests and sandbox drill 035.
5. Rebase onto main, verify with SDD change package.
