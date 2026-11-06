---
change: reuse-one-cat-file-batch-reader-per-repository-root-across-nested-repository-fact-scopes
artifact: testing
---

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
