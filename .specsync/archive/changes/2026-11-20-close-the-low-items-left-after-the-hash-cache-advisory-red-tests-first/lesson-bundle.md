# Lesson bundle: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Close the low items left after the hash-cache advisory, red tests first
- **Kind**: BugFix
- **Specs**: util, hooks, hash_cache, change, commands, cmd_resolve, cmd_stale, cmd_change, cmd_scaffold, cmd_check, validator
- **Paths**: src/util.rs, src/util/local_state.rs, src/hooks.rs, src/hash_cache.rs, src/change/read_cache.rs, src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/collect_requirement_ids.rs, src/commands/mod.rs, src/commands/resolve.rs, src/commands/stale.rs, src/commands/change.rs, src/commands/scaffold.rs, src/commands/doctor.rs, src/validator/requirement_ids.rs, src/change_tests, src/change_tests.rs, tests/integration, tests/integration.rs, CHANGELOG.md, docs, site/src/content/docs
- **Acceptance**: Each red test committed in e334bd8a passes, and each failed on 6.4.0: W1, a linked .specsync/archive/changes is refused naming the folder and no outside name is printed; S1, no hook is written into an existing outside folder the repository's own config names; S2, a journal that does not record the project now at its path is never replayed, and lifecycle commands stop with its path; S3, correct-owner --manifest and scaffold --template refuse a FIFO or an oversized file instead of waiting, still following links; S5, the journal is kept in the user's state folder, not the cache folder; L2, a state folder the current user does not own or that others can write is refused; X1, the scans flag read_dir, WalkDir::new and Git's apply, merge, switch and config unless marked; X2, a hooks path spelled through a link to the work tree is inside it; N2, a state folder that would lie inside the project is refused; H1, a linked worktree's shared hooks folder is made when missing; C1, the change read cache is read and written only through the local state handles; C2, check --only-status keeps a refused spec with its refusal; C3, resolve reports a refused spec; C4, stale counts a refused spec as unmeasurable; C5, a hash cache over the bound is refused, not written. The five history clauses in REQ-util-006, REQ-util-008, REQ-change-162, REQ-hooks-009 and REQ-cmd-check-021 are removed with no change of meaning. macOS verify lane and Linux are green, and change audit passes.

## Evidence

- Verification commit: `25bbd814e63c0e9cede88b0f9a5443cff056ed3e`
- Base commit: `e334bd8abd8f5a2343040d45ca1308cad9e33085`
- Verified by: `specsync check --spec change --spec cmd_change --spec cmd_check --spec cmd_resolve --spec cmd_scaffold --spec cmd_stale --spec commands --spec hash_cache --spec hooks --spec util --spec validator`

## From the change's context.md

# Context

GHSA-7q5v-8jcc-rx54 was fixed in 6.4.0 (#1133, `3d57fa13`) and published on 2026-11-20. What it
left is all low severity:
- sec-review2's re-check of changes 7 and 8 left 13 items: W1, S1 to S5, L1 to L3, X1 to X3, I1.
- corvid-agent's review of #1133 added one (H1).
- #1135 holds the five round-2 Codex P2s on #1133 (C1 to C5).

Leif approved the 6.4.1 scope ("Approve (Recommended)", about 19:51Z on 2026-11-20, relayed by
t4c), with `security/6.4.1-scope.md` as the draft:
- **Fixes:** W1, S1, S2, S3, S5, L2, X1, X2, H1 and C1 to C5, plus the N2 nit.
- **Removals:** the five history clauses SpecSync forced into change 7's requirement texts.
- **Kept as documented:** S4, L1, L3, X3 and I1.
- **His choices:**
  1. A journal that does not record its project, including every 6.4.0 journal, is never
     replayed; lifecycle commands stop with its path.
  2. A typed path still follows links; only the FIFO wait and the size bound are fixed.
  3. The 64 MiB cache bound stays, and a bigger cache is refused.
  4. One change, one review, one re-check by sec-review2.

The red tests were written first (`e334bd8a`), and each failed on 6.4.0.

## From the change's design.md

# Design

- **State folders (S5, N2, L2, S2).**
  - `local_state` gains the user's state folder for the journal, beside the cache folder for the
    caches.
  - `location` refuses a project folder that would lie inside the project (N2).
  - On Unix, opening `specsync/…` checks each folder's owner and mode on the handle (L2).
  - The journal envelope gains `root` (device, inode, canonical path). Replay refuses a journal
    whose root differs or is missing, naming the file. A 6.4.0 journal in the cache folder is
    refused the same way.
- **The read cache (C1)** goes through `LocalState`'s read and write.
- **Hooks (S1, X2, H1).**
  - `reach_git_hooks_dir` adds the common Git directory as a base.
  - It compares resolved paths.
  - It refuses any outside folder unless the person's own config set it.
  - `rev-parse` runs with a clean environment.
- **Listings (W1, X1).**
  - The record-folder listings go through `read_project_dir`.
  - The read scan adds `read_dir(` and `WalkDir::new(`.
  - The Git scan adds `apply`, `merge`, `switch` and `config`.
  - A new scan reports non-Git spawns.
  - Sites that list SpecSync's own state folders, or read Git's config, are marked with the
    reason.
- **Typed paths (S3):** a new `util::read_typed_file` (no-block open, type check, bound), used by
  `--manifest` and `--template`.
- **Refused spec reads (C2 to C4):** `filter_by_status` keeps a spec whose frontmatter read is
  refused; `resolve` prints the refusal; `stale` adds it to the unmeasurable list.
- **Cache bound (C5):** `HashCache::save` refuses JSON over `MAX_CACHE_BYTES` before writing.

## From the change's testing.md

# Testing

| Item | Test |
|------|------|
| W1 | `tests/integration/hardening_641.rs`: `archived_changes_linked_out_of_the_project_are_never_listed` |
| S1 | `hooks_install_writes_no_hook_into_an_existing_folder_the_repository_config_names` |
| S2 | `a_journal_left_for_an_earlier_project_at_the_same_path_is_not_replayed` |
| S3 | `correct_owner_does_not_wait_on_a_fifo_manifest`, `scaffold_does_not_wait_on_a_fifo_template` |
| S5 | `src/change_tests/hardening_641.rs`: `the_journal_is_not_kept_in_the_cache_folder` |
| L2 | `a_state_folder_others_can_write_is_refused` |
| X1 | `project_reads::no_project_file_is_read_outside_the_bounded_read`, `project_writes::no_spawned_git_command_writes_files_unmarked`, and the non-Git spawn scan |
| X2 | `a_hooks_path_spelled_through_a_link_to_the_work_tree_is_inside_it` (macOS) |
| H1 | `hooks_install_in_a_linked_worktree_makes_the_shared_hooks_folder` |
| N2 | `util::local_state::tests::a_project_that_would_hold_its_own_state_folder_is_refused` |
| C1 | `the_change_read_cache_is_never_waited_on` |
| C2 | `only_status_keeps_a_refused_spec_with_its_refusal` |
| C3 | `resolve_reports_a_refused_spec` |
| C4 | `stale_counts_a_refused_spec_as_unmeasurable` |
| C5 | `hash_cache::tests::save_refuses_a_cache_larger_than_the_load_bound` |

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-006 | `src/change_tests/hardening_641.rs`: `the_journal_is_not_kept_in_the_cache_folder`; `src/util/local_state.rs`: `a_project_that_would_hold_its_own_state_folder_is_refused`; `tests/integration/hardening_641.rs`: `a_state_folder_others_can_write_is_refused` |
| REQ-util-008 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`; `tests/integration/project_writes.rs`: `no_spawned_git_command_writes_files_unmarked`, `no_spawned_command_other_than_git_is_unmarked` |
| REQ-util-009 | `src/change_tests/hardening_641.rs`: `the_change_read_cache_is_never_waited_on`; `src/change_tests/fast_archived_reads.rs`: `a_cache_flush_never_shares_its_temporary_file_with_another_writer` |
| REQ-util-010 | `tests/integration/hardening_641.rs`: `archived_changes_linked_out_of_the_project_are_never_listed` |
| REQ-util-011 | `tests/integration/hardening_641.rs`: `correct_owner_does_not_wait_on_a_fifo_manifest`, `scaffold_does_not_wait_on_a_fifo_template` |
| REQ-change-162 | `tests/integration/hardening_641.rs`: `a_journal_left_for_an_earlier_project_at_the_same_path_is_not_replayed`; `src/change_tests/journal_replay_targets.rs`: `a_real_interrupted_write_is_still_recovered` |
| REQ-hooks-009 | `tests/integration/hardening_641.rs`: `hooks_install_writes_no_hook_into_an_existing_folder_the_repository_config_names`, `a_hooks_path_spelled_through_a_link_to_the_work_tree_is_inside_it`, `hooks_install_in_a_linked_worktree_makes_the_shared_hooks_folder` |
| REQ-hash-cache-007 | `src/hash_cache.rs`: `save_refuses_a_cache_larger_than_the_load_bound` |
| REQ-cmd-check-021 | `tests/integration/hardening_641.rs`: `a_state_folder_others_can_write_is_refused` |
| REQ-commands-014 | `tests/integration/hardening_641.rs`: `only_status_keeps_a_refused_spec_with_its_refusal` |
| REQ-cmd-resolve-003 | `tests/integration/hardening_641.rs`: `resolve_reports_a_refused_spec` |
| REQ-cmd-stale-005 | `tests/integration/hardening_641.rs`: `stale_counts_a_refused_spec_as_unmeasurable` |
| REQ-validator-053 | `tests/integration/hardening_641.rs`: `archived_changes_linked_out_of_the_project_are_never_listed`; `src/validator/requirement_ids.rs`: `duplicate_requirement_ids_are_never_looked_for_through_a_specs_link_out` |
| REQ-cmd-change-041 | `tests/integration/hardening_641.rs`: `correct_owner_does_not_wait_on_a_fifo_manifest` |
| REQ-cmd-scaffold-009 | `tests/integration/hardening_641.rs`: `scaffold_does_not_wait_on_a_fifo_template` |

## Where these lessons go

- `specs/util/util.3md`
- `specs/hooks/hooks.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/change/change.3md`
- `specs/commands/commands.3md`
- `specs/cmd_resolve/cmd_resolve.3md`
- `specs/cmd_stale/cmd_stale.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cmd_scaffold/cmd_scaffold.3md`
- `specs/cmd_check/cmd_check.3md`
- `specs/validator/validator.3md`
