---
id: use-current-workflow-fixtures-in-verification-persistence-tests
state: archived
type: bug_fix
base_commit: e9c1cf2d41289d84db4214fe26b42cf2f3ea59ff
---

# Use current-workflow fixtures in verification persistence tests

## Intent

use current-workflow fixtures in verification persistence tests

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- verification persistence tests that assert v2 review or finalize next_action build current-workflow records; workflow_v1_verifying_next_action_names_verify_accept_archive still passes; cargo test exact_and_multiple_verification_persistence_commits_remain_current and scoped_review_persistence_commit_keeps_verification_current pass

## No-spec Rationale

Specs already name v1 Verifying as verify then accept then archive. These tests asserted v2 review and finalize next_action while building records through the v1 helper; switch the fixture to the current workflow so the v1 contract stays intact.
