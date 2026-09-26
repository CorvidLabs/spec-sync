# Lesson bundle — lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Lifecycle commits stage only what the change owns, never every untracked file
- **Kind**: BugFix
- **Specs**: cmd_change, change
- **Paths**: src/commands/change.rs, src/change.rs, src/change_tests.rs, docs/ADOPTING.md
- **Acceptance**: change check --commit and change ship --push never commit an untracked file outside the paths the change owns: an untracked file elsewhere in the tree stays untracked and is absent from every lifecycle commit, and each one left out is listed on standard error. Those commits still carry the change workspace, its archive package, the canonical spec and requirements files its deltas write, the sequence ledger, and every tracked edit, staged through explicit literal pathspecs rather than git add -A. The run_checked_commit doc comment states what is actually committed when the second verification fails, and that error names the materialize commit already made. Regression tests drive both commands over a tree holding an unrelated untracked file and fail if it is committed.

## Evidence

- Verification commit: `f50bfdd57e62925c4cebcd6cb60719907cb5ff63`
- Base commit: `be3d90d8a67b31204623dd48c761e9c25770636f`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

`change check --commit` and `change ship --push` both committed through `git_commit_all` in
`src/commands/change.rs`, which ran `git add -A`. That stages every untracked, non-ignored file
in the project, and `--push` publishes it. In one week it put a private debug zip into a pushed
arcsite commit, and an agent's `.agents/` directory and an `exp2.sh` experiment script into
corvid-bot commits. Nothing warned, because nothing in the lifecycle knew which files were its own.

The same function carried a doc comment on `run_checked_commit` saying nothing is committed unless
verification passes. That holds for the first pass only: when the second pass fails, the
materialize commit is already on the branch.

Constraints a session picking this up needs:

- Verification digests the working tree, tracked and untracked (`project_input_digest` walks
  `git ls-files --cached --others --exclude-standard`). A tracked edit left unstaged would be
  verified and never committed, so tracked edits stay in the commit.
- An untracked file that is left out stays inside that digest. Committing it later keeps the
  evidence current. Removing it stales the evidence, so the warning has to say so.
- `change new` writes `.specsync/workflow-v2-baseline.json` in a freshly adopted project. The
  first test run of the fix left it out, which would have committed a change whose origin anchor
  never reached history. It is a lifecycle ledger and is owned.
- `.specsync/change.lock` and the transaction journal are runtime files. `init` ignores them,
  but a project without that ignore file used to commit the lock through `git add -A`. They are
  neither committed nor reported now.
- The command layer holds no lifecycle policy (`specs/cmd_change/context.md`), so the decision
  about which untracked paths the change owns lives in `src/change.rs`.

Ruled out: owning the change's `affected_paths`. They are prefixes such as `src/` or `.`, and
owning every untracked file under them brings the sweep back. Also ruled out: owning a whole spec
directory, because a stray file dropped beside a spec is not a spec.

## From the change's design.md

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

## From the change's testing.md

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

## Where these lessons go

- `specs/cmd_change/context.md`
- `specs/change/context.md`
