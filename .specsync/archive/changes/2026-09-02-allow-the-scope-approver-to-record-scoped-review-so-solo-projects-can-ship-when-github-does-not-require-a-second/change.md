---
id: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
state: archived
type: bug_fix
base_commit: 6a5a7a7d893fb43515c51514989e5b06674656c4
---

# Allow the scope approver to record scoped review so solo projects can ship when GitHub does not require a second reviewer.

## Intent

Allow the scope approver to record scoped review so solo projects can ship when GitHub does not require a second reviewer.

## Affected Canonical Specs

- `change`
- `agents`
- `cmd_change`

## Acceptance Criteria

- change review accepts the same actor as the definition approver. Solo projects can record review and ship without inventing a second identity. GitHub remains the merge authority for required reviewers. Existing tests that required a distinct reviewer now expect same-actor review to succeed. ADOPTING.md and AGENTS.md no longer tell solo adopters to pick a second identity.

## No-spec Rationale

Not applicable
