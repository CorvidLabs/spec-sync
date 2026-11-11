---
id: change-review-refuses-to-pass-a-change-finalize-would-refuse-on-ownership
state: archived
type: bug_fix
base_commit: 6ef61e6aae6d63b2586d0f9f216c47803dba4d9f
---

# Change review refuses to pass a change finalize would refuse on ownership

## Intent

change review refuses to pass a change finalize would refuse on ownership

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change review with a passing verdict refuses a verifying change whose production delivery inputs no declared module owns, naming the blocker and the send-back and answer recovery, and records nothing; a blocking verdict is still recorded; so a passing review at HEAD implies ship would not refuse on ownership, and the Archive check's change ship advice holds without a SpecSync binary

## No-spec Rationale

Invariant 51 says canonical ownership is checked before a human is asked to act; a passing review is that act and the Archive check names change ship for a change with a passing review at HEAD. Leif chose this fix (option b) on 2026-11-10
