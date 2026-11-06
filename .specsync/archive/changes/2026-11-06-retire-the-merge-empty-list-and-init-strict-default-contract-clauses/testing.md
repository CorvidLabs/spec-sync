---
change: retire-the-merge-empty-list-and-init-strict-default-contract-clauses
artifact: testing
---

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
