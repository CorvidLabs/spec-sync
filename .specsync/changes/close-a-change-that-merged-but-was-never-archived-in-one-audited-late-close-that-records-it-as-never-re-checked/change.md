---
id: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
state: draft
type: feature
base_commit: f7180c3481814c7b05671fb413b1ffec5de829e2
---

# Close a change that merged but was never archived in one audited late close that records it as never re-checked

## Intent

Close a change that merged but was never archived in one audited late close that records it as never re-checked

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- SHIP-12, SHIP-12.c (REQ-change-143): `specsync change close-late <id>... --actor <human> --reason <text>` closes, in one step, a change whose package is committed on the remote default branch, from any state that is not archived or closed late (draft, approved, implementing, verifying, accepted, a partial finalize left in .specsync/changes/, or an accepted package already in the archive); it reads none of the change's delivery inputs and runs no verification, and legacy v1 changes close the same way without change accept.
- SHIP-12.a: the closed package sits in the dated archive with state closed_late and a late-close.json that records the actor, reason, timestamp, the state it was closed from, its workflow version, the remote default ref and tip, the commit that added the package and that commit's date, and verified false; every earlier record in the package is kept byte for byte; change status (text and JSON), ship-status and the change history say it was closed late and never re-checked, and a closed draft also says it was never approved.
- SHIP-12.b: a closed_late change never counts as verified: the archive post-move preflight skips it, so a finalize it blocked before now passes; it is never successor coverage for another change; reopen, review and finalize refuse it; change audit counts it closed.
- SHIP-12.d: close-late refuses an empty --actor or --reason and writes nothing; the actor and reason stay in late-close.json.
- SHIP-12.e: close-late refuses, writing nothing, a change whose package is not committed on the remote default branch, and refuses when no remote default resolves; an unmerged change left in the partial-finalize state is refused and left as it was, and once the change blocking it is closed late its own finalize resumes and archives it.
- SHIP-12.f: several IDs in one call are all checked before anything is written; one refusal writes nothing and names the change refused; --dry-run reports what would close and why and writes nothing.
- Failing tests first: each new test fails on main f7180c34 and passes after the change; no existing gate, bound, timeout or assertion is loosened.

## No-spec Rationale

Not applicable
