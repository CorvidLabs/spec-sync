---
spec: comment.spec.md
---

## User Stories

- As a developer, I want PR comments with clickable links to failing specs so I can navigate directly to the problem
- As a CI operator, I want a single rendering function (`render_check_comment`) that produces consistent output for both `specsync check --format github` and `specsync comment`
- As a marketplace action user, I want identical comment formatting regardless of whether spec-sync runs via the action or a project's own CI

## Acceptance Criteria

- `render_check_comment` produces valid GitHub-flavored markdown with pass/fail header (✅/❌), summary table (specs checked, passed, errors, warnings, file coverage, LOC coverage), grouped error/warning sections, an actionable checklist, an unspecced-files section, and a footer
- When repo and branch are provided, spec links are full GitHub URLs; otherwise plain inline-code markdown
- Errors are classified into actionable categories: missing sections, missing source files, DB table issues, frontmatter problems, dependency issues, schema column issues, and stale file references (with a generic "Review and fix" fallback)
- Warnings are classified into: undocumented export, consumed-by, schema column (with a generic "Review" fallback)
- Errors and warnings are grouped by spec path (`group_by_spec`), preserving insertion order; messages prefixed with `spec/path: ...` have the prefix stripped for the checklist
- The unspecced-files list is truncated to 15 entries with an "...and N more" line
- `detect_branch` returns `Some(branch)` inside a git repo, `None` otherwise
- Both the marketplace GitHub Action (`action.yml`) and project CI workflow (`.github/workflows/ci.yml`) use `specsync comment` to generate identical PR comment output
- Rendered comments never exceed 49,152 bytes, preserve valid UTF-8 when shortened, and append explicit `specsync check --format github` guidance when truncated

## Constraints

- Must produce valid GitHub-flavored markdown
- Must include clickable spec file links when repo/branch are provided
- Unified output pipeline: no separate rendering paths for different integrations
- Must leave headroom below GitHub's 65,536-byte comment limit for integrations that wrap the rendered body

## Out of Scope

- Posting comments (handled by `cmd_comment` / the GitHub Action and CI workflow)
- Interactive or terminal-formatted output
- Public violation-level rendering: the public API is just `render_check_comment` and `detect_branch`. (`SpecViolation` and `render_comment_body` exist only as private test helpers in the `#[cfg(test)]` module, not as part of the module's API.)

### REQ-comment-001

The `comment` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-comment-002

GitHub check comments SHALL accept planned-mapping notices as an explicit renderer input and present them separately from errors and warnings.

Acceptance Criteria

- The canonical `render_check_comment` signature includes the notice collection.
- Planned mappings render in their own section.
- Notice-only results remain passing and do not inflate warning totals.

