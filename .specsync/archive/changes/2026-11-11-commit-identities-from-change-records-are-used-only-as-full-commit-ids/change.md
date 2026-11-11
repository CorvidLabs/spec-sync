---
id: commit-identities-from-change-records-are-used-only-as-full-commit-ids
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Commit identities from change records are used only as full commit ids

## Intent

commit identities from change records are used only as full commit ids

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- A change's base commit, verification and scoped-review commits, and bootstrap and reopening anchors read from records reach Git as revisions only when they are full commit ids, after --end-of-options; a recorded base that is not one makes the coverage gate fall back to the no-record base instead of failing

## No-spec Rationale

Makes behavior match REQ-change-013 (persisted workspace values are validated before use); the change spec only lists a new test file
