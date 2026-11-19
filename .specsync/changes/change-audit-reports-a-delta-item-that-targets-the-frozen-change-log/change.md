---
id: change-audit-reports-a-delta-item-that-targets-the-frozen-change-log
state: approved
type: bug_fix
base_commit: 2cc0897fc00eee7db2b07f5134ba05639af39018
---

# Change audit reports a delta item that targets the frozen Change Log

## Intent

change audit reports a delta item that targets the frozen Change Log

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- (1) change audit reports, for an approved, implementing or verifying change, the Change Log refusal change check gives, prefixed by the change id, and change audit --strict exits 1. (2) An archived change is not judged again. (3) A project with no such item audits as before.

## No-spec Rationale

Not applicable
