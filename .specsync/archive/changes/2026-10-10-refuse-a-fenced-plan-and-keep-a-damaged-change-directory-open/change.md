---
id: refuse-a-fenced-plan-and-keep-a-damaged-change-directory-open
state: archived
type: bug_fix
base_commit: d2448bbbfdabfe37e44aaaceb361d03c762af6c2
---

# Refuse a fenced plan and keep a damaged change directory open

## Intent

Refuse a fenced plan and keep a damaged change directory open

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- A plan cannot be completed from inside a code fence, and a change directory that holds a file but no state.json stays open.

## No-spec Rationale

Not applicable
