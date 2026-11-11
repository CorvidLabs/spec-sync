# Lesson bundle: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The ownership recovery narrows a scope over files the change never touched
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/sdd_version.rs, src/commands/change.rs, src/change_tests/approve_ownership_names_the_spec.rs
- **Acceptance**: ship-status, ship preflight and the review refusal name an unowned production input the change never modified since its base commit with a recovery that narrows affected_paths, and keep the declare-the-owner recovery only for unowned inputs the change modified; without a base commit or git every input keeps the declare recovery

## Evidence

- Verification commit: `f229aad986021436a2008933e179fa7d0e7d0fef`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #1000, found shipping #994: a change approved with an over-wide `affected_paths`
prefix (`src/`) went through approve, check and review, and ship refused it. 127 of the 164
files under the prefix belonged to modules the change never touched. The recovery then said to
declare every owning module, when the right fix was a narrower scope.

Leif's direction on 2026-11-10 (relayed by coordinator t370): rework #1000 to fit #1028. Approve
keeps skipping directory scopes, and finalize handles them. #1034 (#1002) already makes
`change review` refuse a pass while this blocker exists, so the problem surfaces before a human
records a review.

This change fixes the remaining half: `ownership_preflight_blocker` splits unowned inputs by
whether the change modified them: changed since the merge base with the remote default branch
(the base the coverage gate diffs against) or untracked, and not covered by another active change.
Modified ones keep the declare-the-owner recovery. Untouched ones get a recovery that narrows
`affected_paths`, listing the changed paths in scope. Without a full commit id as the change's
base commit, a merge base, a readable set of active changes or a complete spec listing, every
input keeps the declare recovery.

Independent reviews (2026-11-10 and 11) led to these fixes:

- A `base_commit` read from the record could become a Git option. It is now only a validity gate
  and never reaches Git.
- Non-ASCII paths, renames and edits committed before `change new` are counted as modified.
- The blocker hints agree with the recovery.
- A mixed case gets one send-back, and it keeps the `files:` repair for modified inputs no
  module claims.
- The narrowing answer lists the changed paths in scope, deletions included; forgetting a deletion
  fails `change audit --strict` once the deleted file is on the default branch (dogfooded).
- Comparing with `base_commit` counted upstream edits merged into the branch as this change's
  work; the merge base alone does not. A file another active change covers is left to it.

With no remote default branch (so no merge base), nothing is classified, and every input keeps
the declare recovery.

## From the change's design.md

# Design

`ownership_preflight_blocker` keeps one blocker sentence naming every unowned input. An untouched
input's hint names its owner and says to leave it out of `affected_paths`. "Modified" means changed
since the merge base with the remote default branch, the base the coverage gate diffs against, or
untracked, and not covered by another active change. It is computed with NUL-separated
`git diff --name-only --relative --no-renames -z --end-of-options <merge-base>` and
`git ls-files --others --exclude-standard -z`. The change's `base_commit` is not diffed: after the
branch merges or rebases the default branch, that diff also holds upstream edits the change never
made. It is only a gate: it must be a full commit id, or nothing is classified as untouched and
every input keeps the declare recovery. An untouched input whose owners the printed declare answer
already declares is fixed by that answer and is not narrowed.

The recovery has three shapes:

- **Only modified inputs:** the previous declare recovery, unchanged.
- **Only untouched inputs:** a narrowing recovery. `change send-back`, then `change answer <id>
  affected_paths <paths>` listing every changed path inside the current scope, deletions included,
  so a removed file is not dropped from the scope and the shipped change still passes the
  meaningful-path audit. Scopes match as finalize matches them (`path_matches_scope`, so `.` is
  the whole project). When nothing is classified, nothing in the scope changed, a path holds a
  comma, a backslash, a control character or edge whitespace, or the list starts with `-`, the
  answer falls back to a quoted `'<path>,<path>'` placeholder, which keeps the printed command
  honest (#1011).
- **Both:** one send-back carrying the declare answer, any `files:` repair for modified inputs no
  module claims, and the narrowing answer, then one approve, check and review.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
