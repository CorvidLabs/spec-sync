## MODIFIED

### REQUIREMENT REQ-github-001

GitHub helpers SHALL resolve repositories and issue state predictably while redacting credentials
from surfaced failures.

Acceptance Criteria

- Issue reads, listing, and verification use in-process GitHub REST, require an explicit
  `GITHUB_TOKEN`, and do not spawn a `gh` provider process.
- Issue verification preflights repository access once and revalidates access after an apparent
  missing issue before classifying not_found; provider failures are errors.
- One verification batch globally deduplicates at most 100 issue IDs across specs.
- REST issue operations use a 10-second deadline; repository preflight and all fetches share the
  complete 30-second verification deadline.
- Single and listed provider responses require numeric issue identity plus valid title, state,
  labels, and URL; list labels are encoded as query parameters and malformed PR markers fail closed.
- Issue listing rejects raw provider pages above 100 entries before item parsing, including
  pull-request entries. Every raw issue/pull-request item is fully validated before PR filtering:
  marker shape, positive numeric identity, nonempty title, nonempty names for any labels, exact open state, and canonical
  repository/resource/number URL identity must agree, including exact canonical decimal number
  spelling with no leading zeros. Duplicate raw identities within or across
  pages fail even when a duplicate would be filtered as a pull request. Listing follows strict
  encoded `Link` pagination for at most 100 pages and fails on malformed links or a continuing next
  page at the cap instead of returning a truncated batch import. Every next link must retain the
  requested repository issues endpoint and exact open-state, page-size, label, and page semantics.
- `gh` remains available only for the explicit `create_drift_issue` write path.

### SPEC SECTION Public API

#### Exported Functions

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
| `list_issues` | `repo: &str, label: Option<&str>` | `Result<Vec<GitHubIssue>, String>` | List open issues through strict in-process REST pagination; requires `GITHUB_TOKEN` and skips pull requests |
| `create_drift_issue` | `repo: &str, spec_path: &str, errors: &[String], labels: &[String]` | `Result<GitHubIssue, String>` | Create a "Spec drift detected" issue with validation errors |

#### Exported Structs

| Type | Description |
|------|-------------|
| `GitHubIssue` | Issue metadata — `number: u64`, `title: String`, `state: String`, `labels: Vec<String>`, `url: String` |
| `GitHubIssueDetails` | Crate-visible validated issue metadata plus body used by the importer |
| `IssueVerification` | Per-spec result — `spec_path: String`, `valid: Vec<GitHubIssue>`, `closed: Vec<GitHubIssue>`, `not_found: Vec<u64>`, `errors: Vec<String>` |

### SPEC SECTION Invariants

1. Read/list/verify operations require `GITHUB_TOKEN` and execute only in-process REST requests.
2. GitHub issue verification confirms repository access before and after apparent absence.
3. Verification globally deduplicates no more than 100 issue IDs per batch and bounds REST operation
   and complete-batch time.
4. Authentication, repository, transport, timeout, and malformed-provider failures are
   inconclusive errors rather than successful empty or not_found results.
5. Legacy `gh` issue reads fail closed without process spawning; `gh` is reserved for explicit
   issue creation.
6. Issue listing rejects provider pages above 100 entries before parsing any item, strictly
   validates every raw issue/pull-request item as open with exact URL identity before filtering,
   rejects duplicate raw identities within/across pages, paginates at most 100 pages, binds next
   links to the requested repository issues endpoint and query semantics, and rejects malformed
   links and cap truncation.
