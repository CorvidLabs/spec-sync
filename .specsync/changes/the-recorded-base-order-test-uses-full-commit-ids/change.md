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

## Acceptance Criteria

- recorded_base_uses_oldest_change_order_not_hash_order builds its records with full commit ids, so it still proves the oldest change's base is used once only full commit ids are accepted

## No-spec Rationale

Test fixture follow-up to the full-commit-id rule (REQ-change-013); no spec text
