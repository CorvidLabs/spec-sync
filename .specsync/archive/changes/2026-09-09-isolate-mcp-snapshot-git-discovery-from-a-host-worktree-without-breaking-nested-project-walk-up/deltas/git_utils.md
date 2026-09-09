## MODIFIED

### REQUIREMENT REQ-git-utils-005

Every production `git` child spawned from `git_utils` SHALL start with a cleared environment, inherit only an allowlist of PATH/locale/home/temp variables, and set `GIT_TERMINAL_PROMPT=0` and `GIT_OPTIONAL_LOCKS=0`. The default path SHALL NOT set `GIT_CEILING_DIRECTORIES`, so a project whose root is a subdirectory of a git repository is still detected as inside that work tree. Host `GIT_CEILING_DIRECTORIES` SHALL NOT be inherited.

Acceptance Criteria
- `GITHUB_TOKEN`, `GH_TOKEN`, `GIT_DIR`, `GIT_ASKPASS`, `GIT_INDEX_FILE`, `SSH_AUTH_SOCK`, and `GIT_CEILING_DIRECTORIES` are not on the allowlist.
- `is_git_repo` is true for a directory that has no `.git` of its own but sits inside a real repository (nested-project / monorepo-subdir).
- A real repository root is still detected.
- A directory that is not inside any git work tree remains `false`.

### SPEC SECTION Public API

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

**Exported Types**

| Type | Kind | Description |
|------|------|-------------|
| `SpecBaseline` | enum | `Commit(String)` (measurable), `Untracked` (history exists, spec is not in it — drift is genuinely zero), `Missing(MissingHistory)` (drift is UNKNOWN and must never be reported as zero) |
| `MissingHistory` | enum | `NotARepository` or `NoCommits` — why a tree cannot be asked how far a spec has fallen behind |

**Exported Methods**

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| `reason` | `self: MissingHistory` | `&'static str` | Lowercase reason (`not a git repository` / `repository has no commits`), for machine payloads and mid-sentence use |
| `sentence` | `self: MissingHistory` | `&'static str` | Sentence-cased reason (`Not a git repository` / `Repository has no commits`), for the head of a human-readable error line |
| `UnmeasurableSpec` | struct | A spec whose staleness could not be measured at all: path, module name, and each unmeasurable file with its reason. Distinct from a fresh spec — a fresh spec was compared and found current, this one had nothing to compare, and counting it fresh spends it against the up-to-date total on evidence never gathered |
| `StaleInfo` | struct | Staleness summary for a single spec: path, module name, max commits behind, per-file details, and any cited files that no longer exist — a spec is stale on a deletion alone, whatever the threshold, and every renderer needs the cause or a row reads `0 commits behind` with nothing to act on |

### SPEC SECTION Invariants

1. All git commands execute with `current_dir(root)` to ensure correct repository context
2. Functions return safe defaults (None, 0, false) when git is unavailable or commands fail — except that "no history" is never one of those defaults: `spec_baseline` reports it as `Missing`, never as an absent commit
3. `git_commits_since` uses `git rev-list --count {spec_commit}..HEAD -- {source_file}` to count divergence, taking the spec commit hash as a parameter so it is resolved once per spec rather than once per source file
4. `StaleInfo.source_details` only includes files with commits_behind > 0
5. There is no public way to obtain a spec's commit hash without also learning whether the tree has any history: the raw lookup is private, and `SpecBaseline::Commit` is the only value that yields a hash
6. Default `git_cmd` does not set `GIT_CEILING_DIRECTORIES`. `with_discovery_ceiling` is the only way a git child from this module receives a ceiling, and only for the duration of the closure on that thread

### SPEC SECTION Behavioral Examples

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

#### Scenario: Caller-scoped discovery ceiling

- **Given** a git repository at `repo/` and a directory `repo/tmp/snap` with no `.git` of its own
- **When** `with_discovery_ceiling` is called with ceiling `repo/tmp` and `is_git_repo` is called on `repo/tmp/snap` inside the closure
- **Then** returns false. After the closure returns, `is_git_repo` on `repo/packages/foo` is true again

## ADDED

### REQUIREMENT REQ-git-utils-006

`with_discovery_ceiling` SHALL make git children spawned on the calling thread set `GIT_CEILING_DIRECTORIES` to the supplied absolute path after `env_clear`, and SHALL restore the previous slot when the closure returns or unwinds. The default `git_cmd` path SHALL remain ceiling-free.

Acceptance Criteria
- Inside the closure, `is_git_repo` is false for a directory whose only git metadata is a parent work tree above the ceiling.
- After the closure, nested-project walk-up works again on the same thread.
- `GIT_CEILING_DIRECTORIES` is not on the inherit allowlist, so a host-process ceiling cannot leak into default `git_cmd`.
