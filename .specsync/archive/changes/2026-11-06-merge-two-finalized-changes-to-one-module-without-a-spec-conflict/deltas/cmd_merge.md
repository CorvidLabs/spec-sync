## ADDED

### REQUIREMENT REQ-cmd-merge-003

`specsync merge --install` SHALL register the git merge driver for this clone, and `specsync merge-file` SHALL write a clean contract merge or leave a conflict.

Acceptance Criteria
- `--install` appends `merge=specsync` rules for `*.spec.md` and `requirements.md` under the configured specs directory and sets `merge.specsync.driver` to this executable.
- A symlinked `.gitattributes` is refused.
- `merge-file` writes the merged bytes to the current file and exits 0 only when the merge is clean.
- `cmd_merge`'s marker scan is unchanged.
