---
id: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
state: draft
type: bug_fix
base_commit: ad8544e9da79725f499b38ae4d4b3a052be5f807
---

# After approve, next_action names check --commit, and review and ship refuse a verification commit that lacks the change's work

## Intent

After approve, next_action names check --commit, and review and ship refuse a verification commit that lacks the change's work

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- After approve, status and show name specsync change check <id> --commit. change review and change ship refuse when the verification commit does not contain the change's work, and do not bind a review to a commit that lacks that work.

## No-spec Rationale

Not applicable
