---
id: v1-verification-freshness-test-expects-verify-accept-archive-next-action
state: implementing
type: bug_fix
base_commit: 418fa117e78ad1f65f462f3fc96ef0491d0136d2
---

# V1 verification freshness test expects verify accept archive next_action

## Intent

v1 verification freshness test expects verify accept archive next_action

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- verification_freshness_status_and_check_are_environment_independent passes; next_action is the v1 verify/accept/archive string in local, ci, and github environments both before and after the governed input moves

## No-spec Rationale

The v1 Verifying next_action contract already names verify then accept then archive. This integration test used a version-1 sdd.json and change verify, then asserted v2 review/check strings. Update expectations only.
