## MODIFIED

### REQUIREMENT REQ-git-utils-005

Every production `git` child spawned from `git_utils` SHALL start with a cleared environment, inherit only an allowlist of PATH/locale/home/temp variables, and set `GIT_TERMINAL_PROMPT=0` and `GIT_OPTIONAL_LOCKS=0`. It SHALL NOT set `GIT_CEILING_DIRECTORIES`, so a project whose root is a subdirectory of a git repository is still detected as inside that work tree.

Acceptance Criteria
- `GITHUB_TOKEN`, `GH_TOKEN`, `GIT_DIR`, `GIT_ASKPASS`, `GIT_INDEX_FILE`, and `SSH_AUTH_SOCK` are not on the allowlist.
- `is_git_repo` is true for a directory that has no `.git` of its own but sits inside a real repository (nested-project / monorepo-subdir).
- A real repository root is still detected.
- A directory that is not inside any git work tree remains `false`.

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
