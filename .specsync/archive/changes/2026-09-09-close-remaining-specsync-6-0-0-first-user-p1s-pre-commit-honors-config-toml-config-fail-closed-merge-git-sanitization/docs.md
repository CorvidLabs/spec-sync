---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: docs
---

# Docs

- `MIGRATION.md` 5.x → 6.0 step 4: literal v1 close-out (`verify` → `accept` → `archive`, merge-then-archive), commit `.specsync/workflow-v2-baseline.json` and `.specsync/adoption-report.json` after adopt, name both `change check` and that `verify` no longer runs `verification_commands`, and state `adopt` is silent on a committed still-active v1.
- `docs/6-0-confidence-report.md`: #653 is JSON-only fail-closed; MCP sanitization is `git_utils`-only and `merge.rs` was unhardened. After this package, those rows match the tree. C14/C23 stay as the independent verifier scored them.
- CHANGELOG `[6.0.0]` Fixed: one bullet per P1 plus the v1 next-action / `--kind bug-fix` / migration-doc pins.
