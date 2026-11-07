---
change: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to
artifact: plan
---

# Plan

## Approach

Refuse at approve what approve can see. Make ship preflight and ship-status run finalize's ownership resolution over the delivery inputs, so "ready to finalize" is never printed for a change finalize refuses.

## Out of scope

Ownership resolution rules (REQ-change-053), owner corrections, and changing which paths finalize refuses.

## Steps

1. Failing tests, committed red:
   - the approve unit test: a no-spec-change draft over an existing owned file;
   - the integration test: a directory-scoped no-spec-change change, which says ready on main, then the named recovery ships it.
2. Approve: drop the no-spec-change skip; add the `change answer` line for drafts.
3. Read-only ownership preflight; wire it into ship preflight, `ship --dry-run` and ship-status as a blocker with the recovery.
4. Deltas (REQ-change-053, REQ-change-120, REQ-cmd-change-008), spec invariant, CHANGELOG, lessons.
5. `change check --commit`, `change audit --strict`, verify, pre-push.

## Risks

- Approve now refuses no-spec-change drafts over owned production source that it used to accept. Finalize would have refused every one of them, so no change that could ship is lost.
- The preflight costs one ownership resolution per delivery input on ship-status. It reuses evidence the preflight already captures where possible; measure it on spec-sync.

## Constraints consulted

- REQ-change-053 decides ownership.
- The #901/#880 lesson that a named recovery is only right once a test runs it.
- REQ-cmd-change-031's blocker pattern.
- SPEED-2.b.2.
