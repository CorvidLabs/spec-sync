---
spec: git_utils.spec.md
---

## User Stories

- As a command author (stale/report/check/scoring), I want a small set of git query helpers so I can measure spec freshness without reimplementing `git` invocations in each command
- As a maintainer, I want a single source-file's drift to be measured against a precomputed spec commit so iterating a spec's source list does not trigger an N+1 explosion of `git log` calls
- As a user running outside git, I want these helpers to degrade gracefully (safe defaults) rather than panic
- As a command author, I want the type system — not a guard I have to remember to call — to stop me reporting "no drift" on a tree where drift could not be measured

## Acceptance Criteria

- `spec_baseline(root, spec_file)` returns `Commit(full SHA)` for a tracked spec, `Untracked` for a never-committed spec in a repository that HAS history, and `Missing(..)` when the tree has no history at all
- `missing_history(root)` returns `None` when history is usable, `Some(NotARepository)` outside a repository, and `Some(NoCommits)` for an unborn `HEAD`
- No public function returns a bare `Option<String>` commit hash: the two absences must not be expressible as one value
- `git_commits_since(root, spec_commit, source_file)` returns the count of commits touching `source_file` in the range `spec_commit..HEAD`, and `0` when the range is empty or the commit ref is invalid
- `is_git_repo(root)` returns `true` inside a git work tree and `false` otherwise
- `StaleInfo` carries `spec_path`, `module_name`, `max_commits_behind`, and `source_details: Vec<(String, usize)>`
- All commands run with `current_dir(root)`
- Any git command that fails to spawn or returns unparseable output yields the safe default (`None` / `0` / `false`)

## Constraints

- Must not panic on expected error conditions (no `unwrap`/`expect` on git output in library paths)
- Implemented by shelling out to the system `git` binary via `std::process::Command` — no libgit2 / `git2` dependency
- `git_commits_since` takes a precomputed spec commit hash (resolved once per spec) rather than re-resolving it per source file

## Out of Scope

- Any git mutation (commit, tag, push) — these helpers are read-only
- Listing the set of changed files (callers iterate their own known source list)
- Timestamp-based freshness (commit-count distance is the chosen metric)

### REQ-git-utils-001

The `git_utils` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-git-utils-002

Git helpers SHALL expose whether a repository has any history, distinctly from whether a
path is a work tree.

Acceptance Criteria
- A repository with at least one commit reports that it has history.
- A repository with an unborn HEAD reports that it does not, while still reporting as a work tree.
- A path that is not a repository reports neither.

### REQ-git-utils-003

The absence of git history SHALL be representable as a value.

Acceptance Criteria
- "No repository" and "no commits" are distinguishable, not collapsed into one condition.
- The machine and terminal strings match those established by #558, so a reader refactored onto this helper does not change its output.

### REQ-git-utils-004

A shared predicate SHALL answer whether a cited path was known to git at a given commit and is now absent.

Acceptance Criteria
- The predicate distinguishes a deletion, which git can state and name a commit for, from a path git never tracked, whose drift is genuinely unknown.
- Paths are resolved relative to the project root, so a project inside a subdirectory of the repository is answered correctly.
- Every command that answers a staleness question consumes this predicate rather than re-deriving the distinction, so the answers cannot diverge.

