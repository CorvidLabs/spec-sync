---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: requirements
---

# Requirements

Semantic deltas carry the SHALL statements:

- `REQ-hooks-002` (modified): generated pre-commit runs `specsync check`, not `--strict`
- `REQ-hooks-003` first-run init → add-spec → hooks → commit succeeds
- `REQ-config-015` TOML unloadable (garbage, empty, directory, invalid enum) sets `load_error`
- `REQ-merge-003` `unmerged_paths` uses `git_cmd`
- `REQ-git-utils-007` `git_cmd` is `pub(crate)` and drops sentinels
- `REQ-change-100` v1 Verifying next_action is verify → accept → archive
- `REQ-change-101` uncovered-path remediation uses `--kind bug-fix`
