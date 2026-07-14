## ADDED

### REQUIREMENT REQ-comment-002

GitHub check comments SHALL accept planned-mapping notices as an explicit renderer input and present them separately from errors and warnings.

Acceptance Criteria

- The canonical `render_check_comment` signature includes the notice collection.
- Planned mappings render in their own section.
- Notice-only results remain passing and do not inflate warning totals.


## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `render_check_comment` | `total, passed, warnings, errors, all_errors, all_warnings, all_notices, coverage, overall_passed, repo, branch` | `String` | Render the GitHub PR comment with separate error, warning, and planned-mapping notice sections for `specsync check --format github` and `specsync comment` |
| `detect_branch` | `root: &Path` | `Option<String>` | Detect the current git branch name via `git rev-parse` |
