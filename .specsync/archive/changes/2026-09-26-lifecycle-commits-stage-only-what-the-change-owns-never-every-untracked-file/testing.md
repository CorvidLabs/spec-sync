---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-017 | `check_commit_never_commits_an_unrelated_untracked_file`, `ship_push_never_commits_an_unrelated_untracked_file`, `staging_reads_each_porcelain_entry_once_and_stages_only_tracked_edits`, `a_lifecycle_scope_owns_its_subtree_and_not_a_prefix_sibling`, `the_left_out_warning_names_the_files_and_what_to_do` in `src/commands/change.rs` |
| REQ-cmd-change-012 | `lifecycle_commit_raises_a_stale_ledger_before_staging_it` (renamed from `git_commit_all_raises_a_stale_ledger_before_staging_it`; it now drives `git_commit_lifecycle`) |
| REQ-change-102 | `lifecycle_commit_scope_names_exactly_what_the_change_owns` in `src/change_tests.rs` |

## What the regression tests do

- `check_commit_never_commits_an_unrelated_untracked_file` runs `run_checked_commit` on an
  approved change whose workspace is still untracked. The tree holds a tracked delivery edit and
  three strays: `debug-dump.zip`, `.agents/scratch.md` and `exp2.sh`. It requires every stray
  to be absent from all history and still untracked and unmodified. As controls, it requires the
  workspace files, the workflow-v2 baseline `change new` wrote, and the tracked edit to be
  committed, and nothing else except strays and runtime files to be left over.
- `ship_push_never_commits_an_unrelated_untracked_file` runs `check --commit`, `review` and
  `run_ship --push` into a bare remote, with the strays present from before verification. It
  requires no stray in the remote's history, the archive package in the pushed tree, the vacated
  workspace absent from it, and the pushed tip to be the archive commit.

## Discrimination

Run against the same tree with staging put back, and the file compared byte for byte with the
fixed copy after each experiment:

| Tree | `check_commit_…` | `ship_push_…` |
|---|---|---|
| fixed | ok | ok |
| `git_commit_lifecycle` staging with `git add -A` | **FAILED**: `debug-dump.zip was committed` | **FAILED**: `debug-dump.zip was committed` |
| only the archive commit staging with `git add -A` | n/a | **FAILED** |

`staging_reads_each_porcelain_entry_once_and_stages_only_tracked_edits` fails when the rename
source field is not skipped, because the source name is then parsed as an entry and `git add` is
handed a truncated path.

The first run of the check test also caught a defect in the fix itself.
`.specsync/workflow-v2-baseline.json`, which `change new` writes in a freshly adopted project,
was being left out. It is now an owned ledger, and the test asserts it is committed.

## Suite

`fledge lanes run verify`: fmt, `cargo clippy -- -D warnings` and `cargo check` pass; the
full `cargo test` passes 2504 unit and 437 integration tests with 0 failures; release build
passes; `specsync check --strict --require-coverage 100 --force` passes 62/62 specs with 100%
file coverage; the release-candidate test passes. `fledge trust verify` passes.
