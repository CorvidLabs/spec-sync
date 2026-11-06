---
change: retire-the-merge-empty-list-and-init-strict-default-contract-clauses
artifact: research
---

# Research

Call sites at origin/main:

- `detect_conflicted_specs` (src/merge.rs) returns `Option<Vec<PathBuf>>`. `None` comes from
  `unmerged_paths` when Git cannot run, the directory is not a repository, or the query fails.
  `merge_specs` turns `None` into one `MergeResult` with status `Unknown` and the detail "no conflict scan
  was performed. Re-run with --all". `cmd_merge` (src/commands/merge.rs) exits 1 on `Manual` or `Unknown`.
- `results_to_json` writes all four statuses (`resolved`, `manual`, `clean`, `unknown`), as context.md
  already says.
- Tests: `git_discovery_failure_is_unknown_not_empty` (its doc comment says it replaced the safe-noop
  test), `default_merge_without_git_reports_the_missing_precondition`, and
  `unmerged_paths_is_unknown_outside_a_git_repository`. None of them asserts the CLI exit code; the audit
  said so as well.
- `default_policy` (src/change/sdd_version.rs) writes `enabled: false` and
  `require_change_for_meaningful_files: false` for every fresh init, and its comment explains that
  `SddPolicy::default()` stays fail-closed for omitted fields on deserialize. `init` passes an empty
  verification-command list and does not check for Git before writing the policy.
- Tests: `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` (Git fixture),
  `init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout` (no Git), and
  `a_policy_written_before_a_field_existed_still_loads_and_fails_closed` (omitted fields read `true`).
- Every other backticked test name in specs/merge/testing.md and specs/cmd_init/testing.md resolves to a
  declared function. The exceptions are the two stale names and the `init_` prefix.
