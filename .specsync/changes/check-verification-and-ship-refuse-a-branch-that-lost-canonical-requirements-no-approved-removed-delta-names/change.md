---
id: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
state: approved
type: bug_fix
base_commit: 3bebe8b03094b2079630801584b56f94a1e460c5
---

# Check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names

## Intent

check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change check, change verify, change accept, change finalize and change ship refuse, writing nothing, a change on a branch that lost a canonical requirement no approved REMOVED delta names, and name every lost id and its file; a removal an approved REMOVED item of a change on the branch accounts for passes, and so does one the branch merged in from the default branch; a requirement moved between files under its id passes, and a renumbered one is refused under its old id with the permanence hint; change audit reports the same refusal and change ship-status lists it as a blocker; the usual flows pass as before

## No-spec Rationale

Not applicable
