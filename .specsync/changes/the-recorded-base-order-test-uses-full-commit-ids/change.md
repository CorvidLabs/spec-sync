---
id: the-recorded-base-order-test-uses-full-commit-ids
state: implementing
type: bug_fix
base_commit: f3605f2c9a97af46d8f0cdf0aa46825b70acf1c2
---

# The recorded base order test uses full commit ids

## Intent

the recorded base order test uses full commit ids

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- Only the oldest active change's recorded base is considered as the coverage diff base, and one that is not a full commit id falls back to the no-record base (two-record test); the order test uses full commit ids; status evidence shares is_canonical_commit_id

## No-spec Rationale

Test fixture follow-up to the full-commit-id rule (REQ-change-013); no spec text
