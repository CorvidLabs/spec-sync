# Lesson bundle: retire-the-merge-empty-list-and-init-strict-default-contract-clauses

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Retire the merge empty-list and init strict-default contract clauses
- **Kind**: Documentation
- **Specs**: merge, cmd_init
- **Paths**: specs/merge/requirements.md, specs/merge/merge.spec.md, specs/merge/context.md, specs/merge/merge.3md, specs/merge/testing.md, specs/cmd_init/requirements.md, specs/cmd_init/cmd_init.spec.md, specs/cmd_init/testing.md, specs/cmd_init/cmd_init.3md, tests/integration/helpers.rs, tests/integration/commands/specsync_process.rs, CHANGELOG.md, hi/upkeep.md, hi/setup.md, INTENT.md
- **Acceptance**: UPKEEP-3.b: REQ-merge-001 and the top-level acceptance list say a failed Git discovery returns None and yields one Unknown result that names the missing precondition and --all, writes nothing and exits non-zero, and list Unknown among the result statuses; context.md, merge.3md and testing.md no longer call it an empty list or a safe no-op.
- **Acceptance**: SETUP-1.a.1: REQ-cmd-init-004 says a Git repository receives enabled false and require_change_for_meaningful_files false like any other directory and keeps SddPolicy::default() for omitted fields apart from fresh init; the user story and testing.md no longer promise the change workflow on after setup.
- **Acceptance**: UPKEEP-3.b.1 and SETUP-1.a.2: merge_contracts_call_a_failed_git_discovery_unknown and init_contracts_leave_sdd_off_for_git_repositories fail on main and pass after the change, and each also fails when its module testing.md cites a test name that no function declares.
- **Acceptance**: default_merge_outside_git_exits_nonzero_and_names_the_unperformed_scan pins the exit code the reworded REQ-merge-001 states, and fails when the Unknown arm of cmd_merge stops exiting 1.
- **Acceptance**: Runtime behavior is unchanged: no file under src/ changes, and the existing merge and init tests pass unmodified.

## Evidence

- Verification commit: `30c184c248bf15f6a0eb9c178b777c6c2f3db52e`
- Base commit: `560af9f10e608ce4c506612b0d1d2ae13378807d`
- Verified by: `specsync check --spec cmd_init --spec merge`

## From the change's context.md

# Context

The independent 2026-11-05 audit (finding D12, issue #884) found two current contracts that each give two
answers. The runtime is not in question, and the existing tests already back the intended behavior.

1. Merge. REQ-merge-001, the top-level acceptance list, context.md, merge.3md and the testing.md regression
   matrix say a failed default Git discovery returns no files, an empty list, a safe no-op. merge.spec.md
   invariants 16 and 17, its scenario, REQ-merge-002 and REQ-cmd-merge-002 say `None`, one `Unknown` result,
   exit non-zero. #578 made that change. The matrix still names `git_discovery_failure_is_a_safe_noop`, which
   was renamed to `git_discovery_failure_is_unknown_not_empty`. The result-status list also leaves out
   `Unknown`.
2. Init. REQ-cmd-init-004 AC1 promises Git repositories "normal strict SDD defaults", while REQ-cmd-init-005
   AC4, the spec invariants and v12 say fresh init writes `enabled: false` and
   `require_change_for_meaningful_files: false`. The user story still says init "enables verified SDD", and
   testing.md cites `init_enables_sdd_for_new_projects`, which no function declares. The audit points out
   that `SddPolicy::default()` keeping `true` for omitted fields is a separate path that does not qualify
   the Git-init clause.

Leif approved the intent on 2026-11-05 at 20:24 MST, relayed by orc t18: UPKEEP-3.b, UPKEEP-3.b.1,
SETUP-1.a.1 and SETUP-1.a.2. Decision (e): leave `init --help` unchanged. No runtime change.

## From the change's design.md

# Design

| Clause | Was | Becomes |
|---|---|---|
| REQ-merge-001 AC (and the top-level copy) | returns no files, a safe no-op | returns `None`; one `Unknown` result naming the precondition and `--all`; nothing written; exits non-zero (REQ-cmd-merge-002) |
| REQ-merge-001 results AC (and the top-level copy) | `Resolved` / `Manual` / `Clean` | adds `Unknown` |
| merge context.md, merge.3md | returns an empty list | returns `None`; one `Unknown` result (#578) |
| merge testing.md row | empty list; `git_discovery_failure_is_a_safe_noop` | `None` and `Unknown`; the three tests that pin it |
| REQ-cmd-init-004 | SHALL enable Git-dependent coverage; Git gets strict defaults | the same SDD-off values in Git and non-Git directories; `SddPolicy::default()` only fills omitted fields |
| cmd_init user story | init enables verified SDD and detects test commands | layout and source dirs, change workflow off until `change adopt` |
| cmd_init testing.md | cites `init_enables_sdd_for_new_projects` | cites the real tests, plus a REQ-cmd-init-004 evidence line |

Requirement blocks change through the semantic deltas. The top-level lists, companions and lessons are
direct edits inside the declared paths. Historical change-log rows stay as they are.

## From the change's testing.md

# Testing

The runtime does not change. These existing tests pin it and must still pass unmodified:
`git_discovery_failure_is_unknown_not_empty`, `default_merge_without_git_reports_the_missing_precondition`,
`unmerged_paths_is_unknown_outside_a_git_repository`, `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift`,
`init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout` and
`a_policy_written_before_a_field_existed_still_loads_and_fails_closed`.

Contract pins (UPKEEP-3.b.1, SETUP-1.a.2). They read the tracked contracts, not the binary, and both failed on
main with the contracts unchanged:

- `merge_contracts_call_a_failed_git_discovery_unknown`. REQ-merge-001 must state `None`, one `Unknown`
  result and all four statuses. Five retired phrases must be absent from the five merge contract files.
  Every test name that specs/merge/testing.md cites must be declared.
- `init_contracts_leave_sdd_off_for_git_repositories`. REQ-cmd-init-004 must give Git repositories both
  `false` values and keep `SddPolicy::default()` apart. Four retired phrases must be absent from the five
  cmd_init files. Every test name that specs/cmd_init/testing.md cites must be declared.

Characterization: `default_merge_outside_git_exits_nonzero_and_names_the_unperformed_scan` runs the shipped
binary outside a Git repository and asserts exit 1, "conflict scan not performed", the `--all` advice, and
the absence of the no-conflicts line. It passes on main, and it failed under a temporary mutation that made
the `Unknown` arm stop exiting 1; the source was restored.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-merge-001 | the existing `merge::tests` suite (including `git_discovery_failure_is_unknown_not_empty`, `default_merge_without_git_reports_the_missing_precondition`, `all_files_reports_unreadable_specs_as_manual`, `partial_resolution_preview_keeps_all_or_nothing_write_boundary`), `default_merge_outside_git_exits_nonzero_and_names_the_unperformed_scan` (new, mutation-checked), `merge_contracts_call_a_failed_git_discovery_unknown` (contract pin) |
| REQ-cmd-init-004 | `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift`, `init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout`, `a_policy_written_before_a_field_existed_still_loads_and_fails_closed`, `init_contracts_leave_sdd_off_for_git_repositories` (contract pin) |

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| UPKEEP-3.b | `merge_contracts_call_a_failed_git_discovery_unknown`, `default_merge_outside_git_exits_nonzero_and_names_the_unperformed_scan` |
| UPKEEP-3.b.1 | the merge pin failed on main and passes after (recorded) |
| SETUP-1.a.1 | `init_contracts_leave_sdd_off_for_git_repositories` |
| SETUP-1.a.2 | the init pin failed on main and passes after (recorded) |
| Runtime unchanged | `git diff origin/main --stat -- src` is empty; the existing tests above pass unmodified |

## Verification commands

- `fledge lanes run verify` on the pinned 1.89 toolchain (rustup proxy first on PATH)
- `specsync change check <id> --commit`
- `fledge lanes run pre-push`

## Where these lessons go

- `specs/merge/merge.3md`
- `specs/cmd_init/cmd_init.3md`
