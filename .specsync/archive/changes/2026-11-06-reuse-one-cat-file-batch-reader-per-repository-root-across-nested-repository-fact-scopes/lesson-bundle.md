# Lesson bundle: reuse-one-cat-file-batch-reader-per-repository-root-across-nested-repository-fact-scopes

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Reuse one cat-file batch reader per repository root across nested repository-fact scopes
- **Kind**: BugFix
- **Specs**: git_utils
- **Paths**: src/git_utils.rs
- **Acceptance**: A batched read inside a scope nested in an open same-root scope reuses the outer scope's cat-file --batch reader instead of starting a second one (REQ-git-utils-010)
- **Acceptance**: A failing unit test shows two batch children for one root before the fix and one after
- **Acceptance**: Invalidation still closes every frame's reader, so a read after a git write sees the new state
- **Acceptance**: Per-command spawn ceilings in lifecycle_commands_stay_within_git_spawn_ceilings still pass and no count rises

## Evidence

- Verification commit: `008aee5133d75daa3051d3f14f259d9b842036a6`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check --spec git_utils`

## From the change's context.md

# Context

Codex flagged it on #869 (thread on `src/git_utils.rs`, `batched_object`), and it is filed as #891 item (1). Leif asked for it as its own small change (2026-11-05, relayed by orc t18).

`with_batch` looks only at the innermost frame for `root`. A lifecycle command opens a command-wide scope (`cmd_change`), then nests a project-lock scope (`acquire_project_lock`) and read scopes (`begin_change_read_scope`) for the same root. A nested scope that does a batched read starts its own `cat-file --batch` child, even when the outer scope already has one. That nested reader is also dropped when the nested scope closes, so the next outer read starts yet another. REQ-git-utils-010 allows at most one per root.

Constraints: invalidation must still close every frame's reader (a reader can hold a ref snapshot from before a write), and facts behave as before. Nothing outside `src/git_utils.rs` changes.

## From the change's testing.md

# Testing

- `git_utils::tests::a_nested_scope_reuses_the_outer_batch_reader` (new): counts batch starts with `TEST_BATCH_STARTS`. It reads in the outer scope, then a nested same-root scope, then the outer scope again, and expects one start. A first read inside a nested scope followed by an outer read also expects one start. It must fail before the fix.
- Existing `git_utils` tests, including `a_write_inside_the_scope_forgets_every_fact` and `a_batched_head_read_after_a_commit_sees_the_new_commit`, still pass, so invalidation still closes readers.
- `lifecycle_commands_stay_within_git_spawn_ceilings`: per-command spawn counts must not rise. Before: new 29, answer 15, approve 32, check 110, status 53, review 176, finalize 287.
- `fledge lanes run verify` before review.

## Results

- `a_nested_scope_reuses_the_outer_batch_reader` failed before the fix (2 batch starts, expected 1) and passes after.
- All 30 `git_utils` unit tests pass.
- `lifecycle_commands_stay_within_git_spawn_ceilings` passes. Counts: new 29, answer 15, approve 32, check 110, status 53, review 175 (was 176), finalize 285 (was 287).

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-git-utils-010 | `git_utils::tests::a_nested_scope_reuses_the_outer_batch_reader`, `git_utils::tests::batched_reads_equal_per_file_reads_from_one_child`, `git_utils::tests::a_batched_head_read_after_a_commit_sees_the_new_commit` |

## Where these lessons go

- `specs/git_utils/git_utils.3md`
