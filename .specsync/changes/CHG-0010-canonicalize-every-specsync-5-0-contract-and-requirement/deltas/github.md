## ADDED

### REQUIREMENT REQ-github-001

GitHub helpers SHALL resolve repositories and issue state predictably while redacting credentials from surfaced failures.

Acceptance Criteria
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
