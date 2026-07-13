## ADDED

### REQUIREMENT REQ-change-018

Audited reopening SHALL recognize canonical acceptance recorded in current Git history after squash integration or complete later canonical governance.

Acceptance Criteria

- Definition digest, passed evidence, closing approval, stale delivery inputs, actor, and reason remain mandatory.
- An unreachable verification commit is allowed only when current history records acceptance or later recorded canonical changes govern every affected spec and path.
- Arbitrary off-history evidence remains rejected.

## MODIFIED

### SPEC SECTION Invariants

1. Change identifiers and all persisted scope paths remain project-confined.
2. Definition approval binds the complete immutable change definition.
3. Implementation and verification operate against approved effective contracts.
4. Acceptance rechecks dependencies and conflicts immediately before writing.
5. Canonical deltas and closing evidence are written atomically.
6. Requirement identifiers remain unique across canonical, active, and archived truth.
7. Tombstones remain durable across active and archived history.
8. Verification evidence is bound to the tested commit, workspace inputs, and approved contract.
9. Accepted changes remain active until their delivery diff is integrated.
10. Lifecycle mutations hold a project-scoped lock.
11. A no-spec-change declaration never bypasses a declared public contract change.
12. Foreign adoption imports preserve provenance and never overwrite canonical truth.
13. Active semantic deltas are composed into one deterministic effective contract per module.
14. Accepted stale delivery evidence can only return to verification through an explicit human-authored audit event that preserves the prior verification and superseded closing approval.
15. Reacceptance of an already-applied change never reapplies its semantic delta.
16. Reacceptance of an already-applied change requires the definition digest captured by the latest audited reopen event; further definition changes require a new change workspace.
17. Audited reopen accepts unreachable verification commits only when canonical acceptance is recorded in current history or later recorded canonical changes govern every affected contract surface.
