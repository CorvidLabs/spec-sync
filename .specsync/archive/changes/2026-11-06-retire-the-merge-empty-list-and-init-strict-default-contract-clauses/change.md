---
id: retire-the-merge-empty-list-and-init-strict-default-contract-clauses
state: archived
type: documentation
base_commit: 560af9f10e608ce4c506612b0d1d2ae13378807d
---

# Retire the merge empty-list and init strict-default contract clauses

## Intent

Retire the merge empty-list and init strict-default contract clauses

## Affected Canonical Specs

- `merge`
- `cmd_init`

## Acceptance Criteria

- UPKEEP-3.b: REQ-merge-001 and the top-level acceptance list say a failed Git discovery returns None and yields one Unknown result that names the missing precondition and --all, writes nothing and exits non-zero, and list Unknown among the result statuses; context.md, merge.3md and testing.md no longer call it an empty list or a safe no-op.
- SETUP-1.a.1: REQ-cmd-init-004 says a Git repository receives enabled false and require_change_for_meaningful_files false like any other directory and keeps SddPolicy::default() for omitted fields apart from fresh init; the user story and testing.md no longer promise the change workflow on after setup.
- UPKEEP-3.b.1 and SETUP-1.a.2: merge_contracts_call_a_failed_git_discovery_unknown and init_contracts_leave_sdd_off_for_git_repositories fail on main and pass after the change, and each also fails when its module testing.md cites a test name that no function declares.
- default_merge_outside_git_exits_nonzero_and_names_the_unperformed_scan pins the exit code the reworded REQ-merge-001 states, and fails when the Unknown arm of cmd_merge stops exiting 1.
- Runtime behavior is unchanged: no file under src/ changes, and the existing merge and init tests pass unmodified.

## No-spec Rationale

Not applicable
