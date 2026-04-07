---
module: github
version: 1
status: stable
files:
  - src/github.rs
db_tables: []
tracks: [97]
depends_on:
  - specs/parser/parser.spec.md
---

# GitHub

## Purpose

Links spec files to GitHub issues for traceability. Validates `implements` and `tracks` frontmatter fields against actual GitHub issues, fetches issue metadata, and creates drift detection issues when specs fall out of sync.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `detect_repo` | `root: &Path` | `Option<String>` | Auto-detect GitHub repo (`owner/repo`) from git remote URL |
| `resolve_repo` | `config_repo: Option<&str>, root: &Path` | `Result<String, String>` | Resolve repo from config or auto-detect; error if neither available |
| `gh_is_available` | — | `bool` | Check if the `gh` CLI is authenticated and available |
| `fetch_issue_gh` | `repo: &str, number: u64` | `Result<GitHubIssue, String>` | Fetch issue via `gh` CLI (`gh issue view --json`) |
| `fetch_issue_api` | `repo: &str, number: u64` | `Result<GitHubIssue, String>` | Fetch issue via GitHub REST API with `GITHUB_TOKEN` env var |
| `fetch_issue` | `repo: &str, number: u64` | `Result<GitHubIssue, String>` | Fetch issue — tries `gh` CLI first, falls back to REST API |
| `verify_spec_issues` | `repo: &str, spec_path: &str, implements: &[u64], tracks: &[u64]` | `IssueVerification` | Verify all issue references from a spec's frontmatter |
| `create_drift_issue` | `repo: &str, spec_path: &str, errors: &[String], labels: &[String]` | `Result<GitHubIssue, String>` | Create a "Spec drift detected" issue with validation errors |

### Exported Structs

| Type | Description |
|------|-------------|
| `GitHubIssue` | Issue metadata — `number: u64`, `title: String`, `state: String`, `labels: Vec<String>`, `url: String` |
| `IssueVerification` | Verification result — `valid: Vec<GitHubIssue>`, `closed: Vec<GitHubIssue>`, `not_found: Vec<u64>`, `errors: Vec<String>` |

## Invariants

1. `fetch_issue` always tries `gh` CLI first, falls back to REST API only if `gh` is unavailable
2. `fetch_issue_api` requires `GITHUB_TOKEN` environment variable; returns error if unset
3. `fetch_issue_api` uses a 10-second HTTP timeout
4. Issue state is normalized to lowercase (`"open"` / `"closed"`)
5. `create_drift_issue` requires `gh` CLI — no REST API fallback for issue creation
6. `detect_repo` handles both SSH (`git@github.com:`) and HTTPS (`https://github.com/`) remote URLs
7. `resolve_repo` prefers explicit config over auto-detection
8. `verify_spec_issues` classifies each issue as valid (open), closed, not_found, or error

## Behavioral Examples

### Scenario: Verify spec issues

- **Given** a spec with `implements: [42]` and `tracks: [100]`, issue #42 is open, #100 is closed
- **When** `verify_spec_issues` is called
- **Then** returns `valid: [#42]`, `closed: [#100]`, `not_found: []`, `errors: []`

### Scenario: Auto-detect repo from SSH remote

- **Given** git remote URL is `git@github.com:CorvidLabs/spec-sync.git`
- **When** `detect_repo(root)` is called
- **Then** returns `Some("CorvidLabs/spec-sync")`

### Scenario: Create drift issue

- **Given** a spec has validation errors
- **When** `create_drift_issue(repo, path, errors, labels)` is called
- **Then** creates a GitHub issue titled "Spec drift detected: {path}" with error list in body

### Scenario: gh CLI unavailable, API fallback

- **Given** `gh auth status` fails but `GITHUB_TOKEN` is set
- **When** `fetch_issue(repo, 42)` is called
- **Then** falls back to REST API and returns the issue

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No git remote configured | `detect_repo` returns `None` |
| Neither config repo nor git remote | `resolve_repo` returns `Err` |
| `gh` unavailable and no `GITHUB_TOKEN` | `fetch_issue` returns `Err` |
| Issue does not exist (404) | `fetch_issue_api` returns `Err("Issue not found")` |
| Network timeout | `fetch_issue_api` returns `Err` after 10 seconds |
| `gh` CLI not authenticated | `gh_is_available` returns `false` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| (external) | `gh` CLI for authenticated GitHub operations |
| (external) | `ureq` crate for HTTP REST API calls |
| (external) | `serde_json` for parsing JSON responses |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `verify_spec_issues`, `create_drift_issue`, `resolve_repo` via `cmd_check` and `cmd_issues` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-06 | Initial spec for v3.3.0 |
