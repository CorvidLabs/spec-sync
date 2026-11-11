---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: testing
---

# Testing

`the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched` runs on a temp git
project. It fails when the split is disabled. The existing recovery tests, whose unowned
inputs are all modified, are unchanged.

The same test covers names Git would C-quote by default (`core.quotePath`): a committed edit to
`src/billing/tarifa-ñ.ts` and an untracked new `src/billing/recargo-é.ts` both keep the declare
recovery. Each case fails when its Git call drops `-z` (the diff and the untracked-files arm). A
record whose `base_commit` is absent, or is not a full commit id, classifies nothing: every owner
keeps the declare recovery, and the value never reaches Git. That case fails when the base-commit
gate is removed.

Once the change's own commits reach the remote default branch, nothing is classified:
`the_ownership_recovery_declares_every_owner_once_the_change_reached_the_default_branch` pushes
the branch to the default branch, then adds a local commit on top. It fails when the guard for a
merge base that holds the change's own workspace is removed.
`the_ownership_recovery_declares_every_owner_when_head_is_the_merge_base_past_the_base` does the
same with the records never committed, and fails when the guard for HEAD at the merge base is removed.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-008 | unit `the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched`, `the_ownership_recovery_declares_every_owner_once_the_change_reached_the_default_branch`, `the_ownership_recovery_declares_every_owner_when_head_is_the_merge_base_past_the_base`, `the_ownership_recovery_names_a_repair_for_every_unowned_input`; integration `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships` |
| REQ-change-053 | unit `the_ownership_recovery_only_narrows_when_every_unowned_input_is_untouched`, `the_ownership_recovery_declares_every_owner_once_the_change_reached_the_default_branch`, `the_ownership_recovery_declares_every_owner_when_head_is_the_merge_base_past_the_base`, `files_changed_since_base_is_root_relative_and_counts_pre_change_work`, unit `the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched`, `the_ownership_recovery_quotes_every_interpolated_word`, `the_ownership_recovery_leaves_a_sibling_changes_file_to_that_change`, the merged-upstream-edit case in `the_ownership_recovery_only_narrows_when_every_unowned_input_is_untouched`, `the_narrowing_answer_names_every_changed_path_in_scope_or_falls_back_to_the_placeholder` (`.` scope and fallbacks) |
