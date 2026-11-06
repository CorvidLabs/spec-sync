---
module: git_utils
version: 20
status: stable
files:
  - src/git_utils.rs
db_tables: []
tracks: []
depends_on: []
---

# Git Utils

## Purpose

Shared git utility functions for querying repository history. Provides the staleness baseline for a spec, commit distance counting, and git repository detection. Used by the `stale`, `report`, `check`, `lifecycle`, and `scoring` modules to determine spec freshness relative to source file changes. Callers resolve a spec's baseline once via `spec_baseline`, then count divergence per source file via `git_commits_since` to avoid redundant `git log` invocations.

`spec_baseline` is the module's choke point. It replaced an `Option<String>` commit lookup that collapsed two unrelated absences into one `None`: "history exists, this spec simply is not in it" (drift is genuinely zero) and "there is no history at all" (drift is UNKNOWN). Four of the five callers read that `None` as the former and reported every spec current on a tree with no `.git`. The returned enum makes the distinction unspellable-away: every caller, including the next one written, is forced through an exhaustive `match` the compiler checks.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `spec_baseline` | `root: &Path, spec_file: &str` | `SpecBaseline` | Resolve the commit a spec's staleness is measured from, distinguishing an untracked spec from a tree with no history. Probes for missing history only when no commit came back, so a healthy repository still costs one `git log` per spec |
| `missing_history` | `root: &Path` | `Option<MissingHistory>` | Whether the tree has committed history to measure staleness against; `None` means history is usable |
| `git_commits_since` | `root: &Path, spec_commit: &str, source_file: &str` | `usize` | Count commits to source_file since a precomputed spec commit hash |
| `source_was_deleted` | `root: &Path, since: &str, path: &str` | `bool` | Whether a cited path was known to git at `since` and is now absent. The single place that question is answered, so every staleness consumer gives the same verdict: a DELETION is a fact git can name a commit for, distinct from a path git never tracked, whose drift is genuinely unknown. Resolves `<rev>:./<path>` relative to the project root, so a project inside a repository subdirectory is answered correctly |
| `is_git_repo` | `root: &Path` | `bool` | Check if a directory is inside a git work tree |
| `has_commits` | `root: &Path` | `bool` | Whether the repository has any history. An unborn `HEAD` is a work tree by every other test, but nothing can be newer or older than a history that does not exist |
| `with_discovery_ceiling` | `ceiling: &Path, f: impl FnOnce() -> R` | `R` | Run `f` so git children spawned on this thread set `GIT_CEILING_DIRECTORIES` to `ceiling` after `env_clear`. Restores the previous slot on return or unwind. Default `git_cmd` does not pin a ceiling |
| `git_cmd` | `root: &Path` | `Command` | Crate-visible `git` spawn: `env_clear`, PATH/locale/home/temp allowlist, `GIT_TERMINAL_PROMPT=0`, `GIT_OPTIONAL_LOCKS=0`, `current_dir(root)`. Ceiling-free unless `with_discovery_ceiling` is in effect |
| `output_after_child_exits` | `command: &mut Command` | `io::Result<Output>` | Collect stdout and stderr and return when the direct child exits. Stdin is the null device, as with `Command::output`. Both output streams are pipes this process owns. After the child is reaped, one finite snapshot of the bytes already queued is kept. A descendant that keeps writing does not delay the return |
| `status_without_inherited_output` | `command: &mut Command` | `io::Result<ExitStatus>` | Run `command` with stdout and stderr on the null device and return when the direct child exits, so a descendant cannot keep the caller's output pipes open |
| `read_pipe_until_stop` | `reader`, `stop: &AtomicBool`, `wake: RawFd`, `cap: usize`, `overflow: &AtomicBool` | `io::Result<Vec<u8>>` | Unix read of one pipe until EOF or until `stop` is set. `stop` is checked between reads. When `stop` is observed, one `FIONREAD` snapshot of the bytes already queued is drained and the function returns. Past `cap`, further bytes are discarded and `overflow` is set |
| `begin_repository_fact_scope` | `root: &Path` | `RepositoryFactScope` | Open a per-thread repository-fact scope for `root` until the guard drops (REQ-git-utils-009). A scope nested in an open scope for the same root starts from what the outer one knows |
| `memoized_repository_fact` | `root: &Path, flavor: &'static str, args: &[&str]` | `Option<MemoizedGitOutput>` | The answer an open scope for `root` already holds for this ref-, object- or config-only query; never answers while a discovery ceiling is pinned |
| `note_repository_git` | `root: &Path, flavor: &'static str, args: &[&str], output: &MemoizedGitOutput` | `()` | Record a completed git answer in every open scope for `root`, or forget every fact when `args` may write. A process that did not exit with a code is never remembered |
| `invalidate_repository_facts` | — | `()` | Forget every memoized fact and close every batch reader on this thread; called after git writes and external processes |
| `repository_fact_seed` | `root: &Path` | `Option<RepositoryFactSeed>` | The facts this thread's open scopes for `root` hold, to seed read-only worker threads (REQ-git-utils-011); `None` with no open scope, live reads or a pinned discovery ceiling |
| `begin_seeded_repository_fact_scope` | `seed: RepositoryFactSeed` | `RepositoryFactScope` | Open a worker thread's scope starting from `seed`: same fact rules, its own memo, never a batch reader (REQ-git-utils-011) |
| `live_repository_reads` | — | `LiveRepositoryReads` | Make every git read on this thread ask git until the guard drops: no memoized fact and no batch reader answers. Race checks hold one, because they read twice to see another process change the repository |
| `is_read_only_git_invocation` | `args: &[&str]` | `bool` | Whether running `args` cannot change a memoized fact, skipping leading `-c key=value` options |
| `batched_git_read` | `root: &Path, flavor: &'static str, command: impl Fn() -> Command, args: &[&str], max_bytes: usize` | `Option<MemoizedGitOutput>` | Answer `show <rev>:<path>` (with or without a trailing `--`) of a blob and single-path `ls-tree --full-tree -z --name-only` from one `git cat-file --batch` child per scope and flavor, with the bytes and status the per-file read gives (REQ-git-utils-010); `None` for any other shape, a non-blob, an object over `max_bytes` or no open scope |

**Exported Types**

| Type | Kind | Description |
|------|------|-------------|
| `SpecBaseline` | enum | `Commit(String)` (measurable), `Untracked` (history exists, spec is not in it — drift is genuinely zero), `Missing(MissingHistory)` (drift is UNKNOWN and must never be reported as zero) |
| `MissingHistory` | enum | `NotARepository` or `NoCommits` — why a tree cannot be asked how far a spec has fallen behind |
| `OutputWake` | struct | Unix self-pipe, created after the child is spawned, that wakes every reader when the direct child has been reaped. Neither reader consumes the wake byte |
| `RepositoryFactSeed` | struct | A copy of the facts a command's open scopes for one root hold, handed to its read-only worker threads |
| `RepositoryFactScope` | struct | Drop guard for one repository-fact scope; lifecycle commands hold one for the CLI command, the project lock and each read scope |
| `MemoizedGitOutput` | struct | Exit status, stdout and stderr of one git answer, memoized or batched |
| `LiveRepositoryReads` | struct | Drop guard returned by `live_repository_reads`; nests, and reads are live while any is held |

**Exported Methods**

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| `reason` | `self: MissingHistory` | `&'static str` | Lowercase reason (`not a git repository` / `repository has no commits`), for machine payloads and mid-sentence use |
| `sentence` | `self: MissingHistory` | `&'static str` | Sentence-cased reason (`Not a git repository` / `Repository has no commits`), for the head of a human-readable error line |
| `new` | | `io::Result<OutputWake>` | Open the wake pipe with `FD_CLOEXEC` on both ends |
| `raw_read` | `&self` | `RawFd` | Read end both output readers poll |
| `signal` | `&self, stop: &AtomicBool` | | Store `stop` then write one wake byte |
| `UnmeasurableSpec` | struct | A spec whose staleness could not be measured at all: path, module name, and each unmeasurable file with its reason. Distinct from a fresh spec — a fresh spec was compared and found current, this one had nothing to compare, and counting it fresh spends it against the up-to-date total on evidence never gathered |
| `StaleInfo` | struct | Staleness summary for a single spec: path, module name, max commits behind, per-file details, and any cited files that no longer exist — a spec is stale on a deletion alone, whatever the threshold, and every renderer needs the cause or a row reads `0 commits behind` with nothing to act on |

## Invariants

1. All git commands execute with `current_dir(root)` to ensure correct repository context
2. Functions return safe defaults (None, 0, false) when git is unavailable or commands fail — except that "no history" is never one of those defaults: `spec_baseline` reports it as `Missing`, never as an absent commit
3. `git_commits_since` uses `git rev-list --count {spec_commit}..HEAD -- {source_file}` to count divergence, taking the spec commit hash as a parameter so it is resolved once per spec rather than once per source file
4. `StaleInfo.source_details` only includes files with commits_behind > 0
5. There is no public way to obtain a spec's commit hash without also learning whether the tree has any history: the raw lookup is private, and `SpecBaseline::Commit` is the only value that yields a hash
6. Default `git_cmd` does not set `GIT_CEILING_DIRECTORIES`. `with_discovery_ceiling` is the only way a git child from this module receives a ceiling, and only for the duration of the closure on that thread
7. `git_cmd` is crate-visible so other modules (`merge::unmerged_paths`) spawn git through the same cleared-environment allowlist
8. `output_after_child_exits` returns when the direct child exits. Stdin is the null device. Stdout and stderr are pipes this process owns, not the caller's. When the child-exit signal is observed, the reader drains one finite snapshot of the bytes already queued and stops, so a descendant that keeps writing cannot hold the join and the child's last queued bytes are not dropped (SPEED-1, SPEED-1.e). `status_without_inherited_output` runs the child with both streams on the null device, so a descendant cannot keep the caller's stdout or stderr open after the caller exits
9. A repository fact (a `rev-parse`, `symbolic-ref` or `config` read, or a `rev-list`, `log`, `show`, `ls-tree`, `merge-base`, `cat-file`, `diff-tree`, `for-each-ref` or `show-ref` read without stdin) is asked of git at most once per open scope and root. Working-tree and index reads (`ls-files`, `diff`, `diff-files`, `diff-index`, `status`, `check-attr`) are never memoized. Any other git command, and any external process the command runs, forgets every fact and closes every batch reader on the thread. While a `live_repository_reads` guard is held, nothing is answered from memory, so race checks that read before and after still see another process's change
10. A batched read answers only `HEAD` or a full object id with one literal path, gives the bytes and exit status of the per-file read, and leaves trees shown by `git show` and anything over its bound to the per-file read A batched `ls-tree` listing walks the parent trees for the entry and never loads the object it names, so a gitlink or a partial-clone blob is listed as `ls-tree` lists it; a tree this clone does not hold falls back to `ls-tree`. `git config` lookups with scope or format options (`--local`, `-z` and the like) before `--get`, `--get-all` or `--get-regexp` are read-only facts Scopes nested for the same root share one reader per flavor: a nested scope uses the reader an enclosing scope started, and a reader first needed in a nested scope belongs to the outermost scope for that root
11. A worker thread's seeded scope starts from the command's facts, memoizes its own reads, and never starts a `cat-file --batch` reader; no seed is offered while reads are live or a discovery ceiling is pinned (REQ-git-utils-011)

## Behavioral Examples

#### Scenario: Spec not tracked by git, in a repository that has commits

- **Given** a repository with at least one commit and a spec that has never been committed
- **When** `spec_baseline` is called
- **Then** returns `SpecBaseline::Untracked` — there is nothing for the spec to be behind

#### Scenario: No git history at all

- **Given** a directory that is not a git repository, or a repository with an unborn `HEAD`
- **When** `spec_baseline` is called
- **Then** returns `SpecBaseline::Missing(NotARepository)` or `SpecBaseline::Missing(NoCommits)` — the distance is unknown, not zero

#### Scenario: Source file changed after spec

- **Given** a spec last committed at commit A, and a source file with 3 commits after A
- **When** `git_commits_since` is called with commit A's hash
- **Then** returns `3`

#### Scenario: Project root is a subdirectory of the repository

- **Given** a git repository at `repo/` and a specsync project root at `repo/packages/foo` with no `.git` of its own
- **When** `is_git_repo` is called with `repo/packages/foo`
- **Then** returns true, so staleness, reports, and deletion detection still see the parent work tree

#### Scenario: A descendant keeps the child's output pipe open

- **Given** a child that writes `payload` and exits while a grandchild still holds the child's stdout and is blocked
- **When** `output_after_child_exits` is called
- **Then** it returns `payload` while that grandchild is still alive

#### Scenario: A status child must not inherit the caller's stdout

- **Given** a child that records the path of its stdout
- **When** `status_without_inherited_output` runs it
- **Then** the recorded path is the null device

#### Scenario: Caller-scoped discovery ceiling

- **Given** a git repository at `repo/` and a directory `repo/tmp/snap` with no `.git` of its own
- **When** `with_discovery_ceiling` is called with ceiling `repo/tmp` and `is_git_repo` is called on `repo/tmp/snap` inside the closure
- **Then** returns false. After the closure returns, `is_git_repo` on `repo/packages/foo` is true again

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Not a git repository | `is_git_repo` returns false; `missing_history` returns `Some(NotARepository)`; `spec_baseline` returns `Missing` |
| Repository with an unborn `HEAD` | `has_commits` returns false; `missing_history` returns `Some(NoCommits)`; `spec_baseline` returns `Missing` |
| Git not installed | Treated as "not a git repository": `spec_baseline` returns `Missing`, so callers refuse rather than report zero drift |
| Child output reader panics | `output_after_child_exits` returns an error |
| Spawn fails | `output_after_child_exits` and `status_without_inherited_output` return the spawn error |
| File doesn't exist in git history | `spec_baseline` returns `Untracked`; `git_commits_since` returns 0 |

## Dependencies

None (only uses `std::process::Command` for git CLI calls).

## Change Log

| Date | Change |
|------|--------|
| 2026-10-10 | Add `git_utils.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-04-10 | Initial — extracted from cmd_report for shared use by stale, report, and scoring |
| 2026-11-05 | Repository facts are asked once per command and committed objects are read through one `git cat-file --batch` per command (REQ-git-utils-008, REQ-git-utils-009, #861) |
| 2026-06-07 | Replaced `git_commits_between` with `git_commits_since`, which takes a precomputed spec commit hash so callers resolve it once per spec instead of once per source file (eliminates N+1 `git log` calls) |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin: Staleness detection must refuse a repository with no commits instead of reporting every spec current |
| 2026-08-14 | #572: Made `git_last_commit_hash` private and replaced it with `spec_baseline -> SpecBaseline`, plus `missing_history -> Option<MissingHistory>`. The old `Option<String>` conflated "spec not in history" with "no history", and four of five callers read the second as the first |
| 2026-08-14 | CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i: Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension |
| 2026-08-18 | CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness: A staleness answer must not read an unreadable source as freshness |
| 2026-09-09 | close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving: Close the SpecSync 6.0.0 P1 release defects found in overnight proving |
| 2026-09-09 | stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository: Stop check --fix from overwriting fenced Public API examples and restore git discovery for a project inside a repository subdirectory |
| 2026-09-09 | isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up: Isolate MCP snapshot git discovery from a host worktree without breaking nested-project walk-up |
| 2026-09-09 | close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization: Close remaining SpecSync 6.0.0 first-user P1s: pre-commit honors config, TOML config fail-closed, merge git sanitization, and 5.x upgrade docs |
| 2026-11-05 | Never wait on a descendant's open pipe after a Git child exits (#854, #855, SPEED-1, SPEED-1.e) |
| 2026-11-05 | After the child-exit signal, drain one finite snapshot of queued bytes and give the child a null stdin |
| 2026-11-05 | stop-waiting-on-a-descendant-pipe-after-the-git-child-exits: Stop waiting on a descendant pipe after the git child exits |
| 2026-11-05 | cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos: Cut git process spawns per SpecSync command and run the release-candidate suite with cargo nextest so macOS qualification fits 45 minutes |
| 2026-11-06 | #861's repository-fact and batch-read requirements are REQ-git-utils-009 and REQ-git-utils-010. Its archived package names them REQ-git-utils-008 and 009 because #864 took 008 first; the archive is left as reviewed |
| 2026-11-06 | Nested same-root scopes share one `cat-file --batch` reader instead of starting one each (REQ-git-utils-010, #891) |
| 2026-11-06 | reuse-one-cat-file-batch-reader-per-repository-root-across-nested-repository-fact-scopes: Reuse one cat-file batch reader per repository root across nested repository-fact scopes |
| 2026-11-06 | Prefetch workers start from the command's repository facts in a read-only seeded scope (REQ-git-utils-011, S9) |
| 2026-11-06 | archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout: Archived change status no longer walks history per archive or per workflow-v1 change, and read commands take a timeout that names the phase and how to resume |
