---
module: comment
version: 1
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

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `SpecViolation` | A spec violation with path, errors, warnings, and fix suggestions |

### Exported SpecViolation Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `from_result` | `result: &ValidationResult` | `Self` | Build a SpecViolation from a ValidationResult |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `render_check_comment` | `total, passed, warnings, errors, all_errors, all_warnings, coverage, overall_passed, repo, branch` | `String` | Render full GitHub PR comment for `specsync check --format github` |
| `render_comment_body` | `violations, coverage, repo, branch` | `String` | Render PR comment for `specsync comment` subcommand with diff-aware suggestions |
| `detect_branch` | `root: &Path` | `Option<String>` | Detect the current git branch name via `git rev-parse` |

## Invariants

1. When `repo` and `branch` are provided, spec links are full GitHub URLs (`https://github.com/{repo}/blob/{branch}/{path}`); otherwise relative markdown links
2. Error messages are classified into actionable suggestion categories: missing sections, missing source files, DB table issues, frontmatter problems, dependency issues
3. The comment header includes a pass/fail icon and status based on `overall_passed`
4. Coverage metrics (file and LOC percentages) are always included in the summary table
5. `detect_branch` returns `None` if not in a git repository or git command fails

## Behavioral Examples

### Scenario: Render passing check comment

- **Given** 10 specs checked, all passed, 0 errors, 0 warnings
- **When** `render_check_comment(10, 10, 0, 0, &[], &[], &coverage, true, Some("org/repo"), Some("main"))` is called
- **Then** returns markdown with "✅ SpecSync: Passed" header and summary table

### Scenario: Render failing comment with spec links

- **Given** violations with errors pointing to `specs/auth/auth.spec.md` and repo "org/repo" on branch "feat/auth"
- **When** `render_comment_body` is called
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
| types | `CoverageReport`, `ValidationResult` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli | `render_check_comment`, `render_comment_body`, `detect_branch`, `SpecViolation` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-07 | Initial spec |
