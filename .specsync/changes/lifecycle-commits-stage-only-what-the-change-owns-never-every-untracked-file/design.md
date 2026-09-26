---
change: lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file
artifact: design
---

# Design

## Ownership is a domain answer

`change::lifecycle_commit_scope(root, id) -> Result<LifecycleCommitScope, String>` returns two
lists of project-relative paths:

- `owned` holds the untracked paths a lifecycle commit may stage: the active workspace
  `.specsync/changes/<id>` (still owned after `finalize` moves it, so the removal of its tracked
  files is staged), the archive package once it lives there (`find_change_dir`), each affected
  spec's canonical spec file plus the five `CANONICAL_SPEC_COMPANIONS` beside it (resolved through
  `canonical_module_paths`, the resolver materialization writes through), and the lifecycle
  ledgers: `change-sequence.json`, `workflow-v2-baseline.json`, `archive/legacy-baseline.json`,
  `bootstrap.json` and `hashes.json`.
- `runtime` holds `.specsync/change.lock` and `.specsync/change-transaction.json`, which are
  never staged and never reported.

## Staging is a command-layer mechanism

`git_commit_all` becomes `git_commit_lifecycle(root, scope, message) -> LifecycleCommit`. It
still floors the sequence ledger first (REQ-cmd-change-012/013), then calls
`stage_lifecycle_paths`, which reads `git status --porcelain=v1 -z --untracked-files=all -- .`
once and classifies each entry:

| Entry | Action |
|---|---|
| tracked, worktree column not blank | staged |
| tracked, fully staged already | nothing to do |
| unmerged (`U` in either column, `AA`, `DD`) | not staged, so `git commit` still refuses the conflict |
| untracked under `scope.owned` | staged |
| untracked under `scope.runtime` | ignored |
| any other untracked file | left out and returned |

Staging runs `git --literal-pathspecs add -- <paths>` in batches of 200, so a filename containing
glob or `:` magic is taken literally. Porcelain paths are relative to the repository top level, so
the `rev-parse --show-prefix` prefix is stripped before matching and staging. A project nested in
a larger repository then stages its own paths and only those. Rename and copy entries carry their
source as a second NUL field, in either column, and it is skipped. The index is not reset, so
anything the author already staged is committed as before.

`LifecycleCommit { committed, left_out }` lets the callers disclose and word their output
truthfully:

- `run_checked_commit` warns once per run on stderr, listing at most 20 paths and summarizing the
  rest. It adds that removing a listed file stales the recorded verification and names the command
  that re-records it. When the second verification fails after a materialize commit was made, the
  error names that commit and the resume command. The commit is not rewound.
- `ship_commit_and_push_archive` warns the same way before pushing, without the re-check line,
  because archived evidence is not recomputed against the live tree.

## Why not rewind on a second-pass failure

The alternative was to make the doc comment true by running `git reset --soft` to the pre-commit
HEAD. The second pass verifies the same content as the first, so a failure there is rare, and the
materialize commit left behind is a valid intermediate state that a re-run completes. Moving a
branch tip back on the author's behalf is the less safe choice, so the comment now describes the
behavior and the error names the commit.
