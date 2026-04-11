---
spec: comment.spec.md
---

## User Stories

- As a developer, I want PR comments with clickable links to failing specs so I can navigate directly to the problem
- As a CI operator, I want a single rendering function (`render_check_comment`) that produces consistent output for both `specsync check --format github` and `specsync comment`
- As a marketplace action user, I want identical comment formatting regardless of whether spec-sync runs via the action or a project's own CI

## Acceptance Criteria

- `render_check_comment` produces valid GitHub-flavored markdown with pass/fail header, summary table, coverage metrics, and actionable error suggestions
- When repo and branch are provided, spec links are full GitHub URLs; otherwise relative markdown links
- Errors are classified into actionable categories: missing sections, missing source files, DB table issues, frontmatter problems, dependency issues
- `detect_branch` returns `Some(branch)` inside a git repo, `None` otherwise
- Both the marketplace GitHub Action (`action.yml`) and project CI workflow (`.github/workflows/ci.yml`) use `specsync comment` to generate identical PR comment output

## Constraints

- Must produce valid GitHub-flavored markdown
- Must include clickable spec file links when repo/branch are provided
- Unified output pipeline: no separate rendering paths for different integrations

## Out of Scope

- Posting comments (handled by `cmd_comment`)
- Interactive or terminal-formatted output
- Violation-level rendering (`SpecViolation`, `render_comment_body` were removed in the unified pipeline refactor)
