---
id: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
state: draft
type: feature
base_commit: 40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56
---

# Stop waiting on a descendant pipe after the git child exits

## Intent

stop waiting on a descendant pipe after the git child exits

## Affected Canonical Specs

- `git_utils`
- `change`
- `changelog`
- `cmd_change`
- `cmd_diff`
- `cmd_lifecycle`
- `comment`
- `github`
- `hooks`
- `merge`

## Acceptance Criteria

- A Git read returns the child's own bytes while a descendant still holds that child's output pipe. A status Git child does not receive the caller's stdout. The existing deadline, output cap, and stdin-writer join checks still pass. SPEED-1, SPEED-1.a, and SPEED-1.e.

## No-spec Rationale

Not applicable
