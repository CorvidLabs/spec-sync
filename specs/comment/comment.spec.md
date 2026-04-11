---
module: comment
version: 2
status: stable
files:
  - src/comment.rs
db_tables: []
tracks: [140]
depends_on:
  - specs/types/types.spec.md
---

# Comment

## Purpose

GitHub PR comment formatting with spec links and actionable suggestions. Produces GitHub-flavored markdown output designed for posting as PR comments, including direct links to spec files, actionable checklists, and diff-aware suggestions for updating specs.

**Unified output pipeline**: Both the marketplace GitHub Action (`action.yml`) and the project's own CI workflow (`.github/workflows/ci.yml`) use `specsync comment` to generate PR comments. This ensures identical output regardless of how spec-sync is invoked. The action captures stdout from `specsync comment` and posts it via `gh api`; the CI workflow captures it via `cargo run -- comment` and posts via `actions/github-script`.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `render_check_comment` | `total, passed, warnings, errors, all_errors, all_warnings, coverage, overall_passed, repo, branch` | `String` | Render full GitHub PR comment for `specsync check --format github` and `specsync comment` |
| `detect_branch` | `root: &Path` | `Option<String>` | Detect the current git branch name via `git rev-parse` |

## Invariants

1. When `repo` and `branch` are provided, spec links are full GitHub URLs (`https://github.com/{repo}/blob/{branch}/{path}`); otherwise relative markdown links
2. Error messages are classified into actionable suggestion categories: missing sections, missing source files, DB table issues, frontmatter problems, dependency issues
3. The comment header includes a pass/fail icon and status based on `overall_passed`
4. Coverage metrics (file and LOC percentages) are always included in the summary table
5. `detect_branch` returns `None` if not in a git repository or git command fails
6. The marketplace GitHub Action (`action.yml`) and project CI workflow (`.github/workflows/ci.yml`) both invoke `specsync comment` to produce identical PR comment output

## Behavioral Examples

### Scenario: Render passing check comment

- **Given** 10 specs checked, all passed, 0 errors, 0 warnings
- **When** `render_check_comment(10, 10, 0, 0, &[], &[], &coverage, true, Some("org/repo"), Some("main"))` is called
- **Then** returns markdown with "✅ SpecSync: Passed" header and summary table

### Scenario: Render failing check comment with errors

- **Given** 10 specs checked, 8 passed, 2 with errors pointing to `specs/auth/auth.spec.md` and repo "org/repo" on branch "feat/auth"
- **When** `render_check_comment` is called with errors
- **Then** error lines include clickable GitHub links to the spec file

### Scenario: Detect branch

- **Given** a git repository on branch `feat/new-module`
- **When** `detect_branch(root)` is called
- **Then** returns `Some("feat/new-module")`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Not in a git repository | `detect_branch` returns `None` |
| No repo/branch provided | Spec links use relative markdown format instead of GitHub URLs |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `CoverageReport` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli | `render_check_comment`, `detect_branch` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-11 | Documented unified output pipeline: marketplace action and CI workflow both use `specsync comment` for identical PR comments |
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-07 | Initial spec |
