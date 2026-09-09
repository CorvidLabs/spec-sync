---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: tasks
---

# Tasks

- [x] Generated pre-commit hook honors config enforcement (`specsync check`, not `--strict`).
- [x] TOML load goes through `parse_config_content_checked`; empty file, directory, invalid enum, garbage all set `load_error`.
- [x] `git_cmd` is crate-visible; `unmerged_paths` uses it.
- [x] v1 Verifying next_action names verify → accept → archive; `--kind bug-fix`.
- [x] MIGRATION.md v1 close-out (F1/F4/F5/F6) and confidence-report row corrections.
- [x] Fail-then-pass tests for each P1.
- [x] Definition approved by 0xLeif; scoped check, review, and same-PR finalization follow on the PR.
