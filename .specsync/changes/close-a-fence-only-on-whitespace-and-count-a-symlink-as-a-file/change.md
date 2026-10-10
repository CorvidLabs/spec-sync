---
id: close-a-fence-only-on-whitespace-and-count-a-symlink-as-a-file
state: implementing
type: bug_fix
base_commit: 7713e417e406a53593381e7f934135fa95d163e1
---

# Close a fence only on whitespace and count a symlink as a file

## Intent

Close a fence only on whitespace and count a symlink as a file

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- A code fence closes only when the marker is followed by whitespace, and a symlink keeps a state-less change directory open.

## No-spec Rationale

Not applicable
