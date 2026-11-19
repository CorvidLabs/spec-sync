---
id: fix-the-four-pr-1086-review-findings-on-history-fragments
state: verifying
type: bug_fix
base_commit: 069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d
---

# Fix the four PR 1086 review findings on history fragments

## Intent

Fix the four PR 1086 review findings on history fragments

## Affected Canonical Specs

- `change`
- `hash_cache`

## Acceptance Criteria

- Every valid change id round-trips through its fragment; a fragment swapped for a same-bytes symlink is re-validated instead of replaying a cached pass; a delta item that targets the Change Log is refused at approve, check, verify and accept; a reformatted legacy row plus a fragment is refused as a double record and a row for one-session does not count for session

## No-spec Rationale

Not applicable
