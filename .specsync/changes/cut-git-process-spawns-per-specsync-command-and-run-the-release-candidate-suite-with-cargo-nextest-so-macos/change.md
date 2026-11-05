---
id: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
state: draft
type: feature
base_commit: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
---

# Cut git process spawns per SpecSync command and run the release-candidate suite with cargo nextest so macOS qualification fits 45 minutes

## Intent

cut git process spawns per SpecSync command and run the release-candidate suite with cargo nextest so macOS qualification fits 45 minutes

## Affected Canonical Specs

- `git_utils`
- `change`
- `cmd_change`

## Acceptance Criteria

- Within one SpecSync command, git is asked each repository fact the command does not itself change (HEAD commit, show-prefix, work-tree and shallow state, origin default branch, index location, fsmonitor and core settings) at most once, and command output is byte-identical to before.
- A memoized fact is dropped after the command itself commits, writes the index or moves a ref, so the next read asks git again.
- Committed file contents and tree listings that one command reads at fixed commits come from one `git cat-file --batch` process per repository instead of one `git show` or `git ls-tree` per file.
- A test bounds git process spawns per lifecycle command, and the slowest lifecycle integration test (`eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci`) spawns at most 40% of its 15,563-process baseline.
- The release-candidate lane runs the same test set with a checksum-pinned `cargo-nextest`; no test is removed, ignored, filtered or loosened, and the two tests that are `#[ignore]` today stay the only ignored ones.
- Before and after timings and spawn counts are recorded in `testing.md`.

## No-spec Rationale

Not applicable
