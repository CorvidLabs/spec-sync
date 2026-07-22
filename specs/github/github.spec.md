---
module: github
version: 9
status: stable
files:
  - src/github.rs
db_tables: []
tracks: [102]
depends_on:
  - specs/parser/parser.spec.md
---

# GitHub

## Purpose

Links spec files to GitHub issues for traceability. Validates `implements` and `tracks` frontmatter
fields against actual GitHub issues, fetches issue metadata, and creates drift detection issues
when specs fall out of sync. Also defines the maintained composite GitHub Action distribution
contract: immutable exact-version refs and a verified floating major compatibility ref whose
default binary follows the promoted stable release. Hosted JavaScript verification uses one exact
supported Bun runtime across site deployment, site CI, and VS Code extension CI.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `detect_repo` | `root: &Path` | `Option<String>` | Auto-detect GitHub repo (`owner/repo`) from git remote URL |
| `resolve_repo` | `config_repo: Option<&str>, root: &Path` | `Result<String, String>` | Resolve repo from config or auto-detect; error if neither available |
| `gh_is_available` | — | `bool` | Check whether the `gh` CLI is authenticated for the explicit issue-creation write path |
| `fetch_issue_gh` | `repo: &str, number: u64` | `Result<GitHubIssue, String>` | Reject legacy `gh` issue reads without spawning a provider process |
| `fetch_issue_api` | `repo: &str, number: u64` | `Result<GitHubIssue, String>` | Fetch issue via GitHub REST API with `GITHUB_TOKEN` env var |
| `fetch_issue_details` | `repo: &str, number: u64` | `Result<GitHubIssueDetails, String>` | Crate-visible typed issue/body read for importers, with explicit token, repository preflight, and 404 revalidation |
| `fetch_issue` | `repo: &str, number: u64` | `Result<GitHubIssue, String>` | Fetch one issue through in-process REST; requires `GITHUB_TOKEN` |
| `verify_spec_issues` | `repo: &str, spec_path: &str, implements: &[u64], tracks: &[u64]` | `IssueVerification` | Verify all issue references from a spec's frontmatter |
| `verify_issue_batch` | `repo: &str, references: &[(String, Vec<u64>, Vec<u64>)]` | `Vec<IssueVerification>` | Crate-visible in-process REST verification for one globally deduplicated/capped project batch |
| `list_issues` | `repo: &str, label: Option<&str>` | `Result<Vec<GitHubIssue>, String>` | List open issues through in-process REST; requires `GITHUB_TOKEN` and skips pull requests |
| `create_drift_issue` | `repo: &str, spec_path: &str, errors: &[String], labels: &[String]` | `Result<GitHubIssue, String>` | Create a "Spec drift detected" issue with validation errors |

### Exported Structs

| Type | Description |
|------|-------------|
| `GitHubIssue` | Issue metadata — `number: u64`, `title: String`, `state: String`, `labels: Vec<String>`, `url: String` |
| `GitHubIssueDetails` | Crate-visible validated issue metadata plus body used by the importer |
| `IssueVerification` | Per-spec result — `spec_path: String`, `valid: Vec<GitHubIssue>`, `closed: Vec<GitHubIssue>`, `not_found: Vec<u64>`, `errors: Vec<String>` |

## Invariants

1. `fetch_issue`, `list_issues`, and issue verification use only in-process GitHub REST; none
   launches a `gh` provider process.
2. Every read/list/verify path requires an explicit `GITHUB_TOKEN`; authenticated `gh` state is
   not used as a fallback.
3. Every REST operation uses a 10-second deadline; repository preflight plus all batch fetches share
   one 30-second deadline over at most 100 globally deduplicated issue IDs.
4. Issue state is normalized to lowercase (`"open"` / `"closed"`)
5. `create_drift_issue` requires `gh` CLI — no REST API fallback for issue creation
6. `detect_repo` handles both SSH (`git@github.com:`) and HTTPS (`https://github.com/`) remote URLs
7. `resolve_repo` prefers explicit config over auto-detection
8. Issue verification preflights repository access once and revalidates access after every apparent
   missing issue before classifying not_found; repository, authentication, transport, timeout, and
   malformed-provider failures remain errors.
9. `gh` is invoked only by the explicit `create_drift_issue` write path; legacy
   `fetch_issue_gh` fails closed without spawning it.
10. Issue listing strictly follows `Link` pagination for at most 100 pages. Every next link must
    retain the requested `/repos/{owner}/{repo}/issues` endpoint and the exact open-state,
    100-item, label, and next-page query semantics; malformed or redirected links, duplicate issue
    numbers, and a continuing page after the cap fail instead of truncating.
11. Action defaults and maintained consumer pins advance to an exact release version only through
    an accepted release change, and floating-ref promotion waits for supported-platform
    verification of the exact-version artifacts.

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

### Scenario: Authenticated gh does not authorize reads

- **Given** `gh auth status` succeeds but `GITHUB_TOKEN` is unset
- **When** `fetch_issue(repo, 42)` is called
- **Then** returns a token-required error without launching `gh issue view`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No git remote configured | `detect_repo` returns `None` |
| Neither config repo nor git remote | `resolve_repo` returns `Err` |
| No `GITHUB_TOKEN` | Read, list, and verification paths return an actionable error without consulting `gh` credentials |
| Issue does not exist (404) | Returns not-found only after repository access is revalidated within the operation deadline |
| Network timeout | `fetch_issue_api` returns `Err` after 10 seconds |
| `gh` CLI not authenticated | `gh_is_available` returns `false` |
| Repository missing or inaccessible | Verification is inconclusive; never reported as issue not_found |
| More than 100 unique issue IDs | Verification fails before provider access |
| Duplicate issue IDs across specs | Provider is queried once per unique issue in the batch |
| Malformed REST response | Strict issue verification records an inconclusive provider error; never successful empty verification |
| Duplicate issue across list pages | Entire listing fails; duplicates are not silently removed |
| Issue listing still has a next page after 100 pages | Entire listing fails instead of truncating |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| (external) | GitHub REST for reads/listing/verification; `gh` only for explicit issue creation |
| (external) | `ureq` crate for HTTP REST API calls |
| (external) | `serde_json` for parsing JSON responses |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `verify_spec_issues`, `create_drift_issue`, `resolve_repo` via `cmd_check` and `cmd_issues` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out of scope sections |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-17 | CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur: Prepare the SpecSync 5.1.1 stabilization release from merged PR #387: bump accurate release metadata and changelog, update the GitHub Action default to 5.1.1, document and validate the floating v5 compatibility ref, verify all release artifacts and supported installation paths, and define fail-closed publication and rollback boundaries |
| 2026-07-17 | Hardened release validation for inline security guidance and runner-local candidate mirrors |
| 2026-07-20 | CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c: Prepare the SpecSync 5.2.0 feature release: bump accurate release metadata and changelog, update the GitHub Action default to 5.2.0, document the native migrate 5.0 ledger backfill, batch correct-owner, inert registry stub tolerance, squash-merged archive trust, and legacy archive repair, verify all release artifacts and supported installation paths, and define fail-closed publication and rollback boundaries |
| 2026-07-22 | CHG-0063: Fail closed on inaccessible repositories/provider failures and bound globally deduplicated issue verification |
| 2026-07-22 | CHG-0063 follow-up: Move issue reads, listing, and verification to explicit-token in-process REST, strictly parse encoded issue listings, and disable `gh` read providers |
| 2026-07-22 | CHG-0063 review fix: Bind every pagination link to the requested repository issues endpoint and query semantics |
