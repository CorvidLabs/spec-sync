---
spec: github.spec.md
---

## User Stories

- As a developer, I want spec-sync to auto-detect my GitHub repo from git remotes so that I don't need to configure it manually
- As a team lead, I want to configure the repo explicitly in config so that auto-detection doesn't pick the wrong repo
- As a developer, I want to verify that `implements` and `tracks` frontmatter references actual open GitHub issues so that specs stay linked to real work
- As a developer, I want to be notified when referenced issues are closed so that I can update spec requirements that may no longer be valid
- As a team lead, I want drift detection issues auto-created when specs fall out of sync so that teams are notified of documentation debt
- As a CI operator without `gh` CLI, I want to use `GITHUB_TOKEN` for API access so that checks work in headless environments
- As a developer, I want `gh` CLI tried first before falling back to REST API so that I get the fastest, most reliable access with my existing auth

## Acceptance Criteria

- `detect_repo` extracts `owner/repo` from SSH (`git@github.com:owner/repo.git`), HTTPS (`https://github.com/owner/repo.git`), and `http://github.com/...` remote URLs; the trailing `.git` is optional
- `resolve_repo` prefers explicit config repo over auto-detected repo; returns error if neither is available
- `gh_is_available` returns true only when `gh auth status` succeeds (CLI is installed and authenticated)
- `fetch_issue` tries `gh` CLI first, falls back to REST API only if `gh` is unavailable
- `fetch_issue_api` requires `GITHUB_TOKEN` environment variable; returns clear error if unset
- `fetch_issue_api` uses a 10-second HTTP timeout; returns error on network failure
- Issue state is normalized to lowercase (`"open"` / `"closed"`) regardless of API response format
- `verify_spec_issues` classifies each issue as valid (open), closed, not_found, or error with detailed messages
- `create_drift_issue` requires `gh` CLI — no REST API fallback for issue creation
- `create_drift_issue` creates issue titled "Spec drift detected: {path}" with formatted error list in body
- Drift issues are created with configurable labels (default `["spec-drift"]`, set via `github.drift_labels`)
- `list_issues` lists open issues (optionally filtered by label), preferring `gh` CLI and falling back to the REST API; the REST path skips pull requests
- Auth tokens (`GITHUB_TOKEN`) are redacted from REST request error messages via `redact_token` before being surfaced (defense-in-depth)

## Constraints

- Must support both `gh` CLI (preferred) and direct REST API (fallback) for maximum compatibility
- Must handle unauthenticated or rate-limited scenarios gracefully with actionable error messages
- Must not panic on provider failure — return `Result` with descriptive error message
- HTTP timeout must be enforced even if GitHub API hangs (`ureq` global timeout: 10s for single issues, 15s for `list_issues`)
- Must not require write access to repo for read-only operations (issue verification)
- SSH and HTTPS remote URL formats must both be supported for auto-detection

## Out of Scope

- Creating issues via REST API (only `gh` CLI is supported for creation)
- Updating or closing existing GitHub issues
- Commenting on issues from spec-sync
- Support for GitHub Enterprise Server (github.com only)
- Webhook-based real-time issue monitoring
- Caching issue metadata across runs
- Support for PR references (only issues)
