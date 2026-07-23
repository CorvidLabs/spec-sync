---
spec: github.spec.md
---

## User Stories

- As a developer, I want spec-sync to auto-detect my GitHub repo from git remotes so that I don't need to configure it manually
- As a team lead, I want to configure the repo explicitly in config so that auto-detection doesn't pick the wrong repo
- As a developer, I want to verify that `implements` and `tracks` frontmatter references actual open GitHub issues so that specs stay linked to real work
- As a developer, I want to be notified when referenced issues are closed so that I can update spec requirements that may no longer be valid
- As a team lead, I want drift detection issues auto-created when specs fall out of sync so that teams are notified of documentation debt
- As a CI operator, I want issue reads to require an explicit `GITHUB_TOKEN` so authorization is deterministic in headless environments
- As a security reviewer, I want read/list/verify operations to avoid `gh` provider subprocesses so project checks cannot inherit an escapable process tree

## Acceptance Criteria

- `detect_repo` extracts `owner/repo` from SSH (`git@github.com:owner/repo.git`), HTTPS (`https://github.com/owner/repo.git`), and `http://github.com/...` remote URLs; the trailing `.git` is optional
- `resolve_repo` prefers explicit config repo over auto-detected repo; returns error if neither is available
- `gh_is_available` returns true only when `gh auth status` succeeds (CLI is installed and authenticated)
- `fetch_issue`, `list_issues`, and issue verification use in-process GitHub REST and never launch a `gh` provider process
- Every issue read/list/verify path requires `GITHUB_TOKEN`; authenticated `gh` state is not a fallback
- `fetch_issue_api` uses a 10-second HTTP timeout; returns error on network failure
- Issue state is normalized to lowercase (`"open"` / `"closed"`) regardless of API response format
- `verify_spec_issues` classifies each issue as valid (open), closed, not_found, or error with detailed messages
- Issue verification preflights repository access once, revalidates access after an apparent
  missing issue, classifies only confirmed absent issues as not_found, and treats provider failures
  as errors.
- One verification batch deduplicates issue IDs across specs, accepts at most 100 unique IDs, and
  enforces a 10-second REST operation and 30-second complete repository-preflight/fetch deadline.
- `create_drift_issue` is the only issue operation that invokes `gh`; no REST write fallback is provided
- `create_drift_issue` creates issue titled "Spec drift detected: {path}" with formatted error list in body
- Drift issues are created with configurable labels (default `["spec-drift"]`, set via `github.drift_labels`)
- `list_issues` lists every open issue through in-process REST, requires `GITHUB_TOKEN`, skips pull
  requests, rejects any provider page above 100 entries before item parsing, follows strict GitHub
  `Link` pagination for at most 100 pages, and fails on malformed links, duplicate issue numbers,
  or a page-limit truncation. Pull-request entries count toward the provider-page bound and every
  raw item is validated before PR filtering: marker shape, positive identity, nonempty title,
  nonempty names for any labels, exact
  open state, and canonical repository/resource/number `html_url` identity must agree. Duplicate
  raw identities within or across pages fail even if a duplicate is a filtered pull request. Each
  next link must retain the requested repository issues endpoint and exact `state=open`,
  `per_page=100`, label, and page query semantics.
- Auth tokens (`GITHUB_TOKEN`) are redacted from REST request error messages via `redact_token` before being surfaced (defense-in-depth)

## Constraints

- Read/list/verify operations must use in-process REST; `gh` is reserved for explicit issue-creation writes
- Must handle unauthenticated or rate-limited scenarios gracefully with actionable error messages
- Must not panic on provider failure — return `Result` with descriptive error message
- Every REST operation uses a 10-second global timeout; verification additionally has a 30-second batch deadline
- Read/list/verify operations must not spawn a `gh` provider process on any platform.
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

### REQ-github-001

GitHub helpers SHALL resolve repositories and issue state predictably while redacting credentials from surfaced failures.

Acceptance Criteria
- `detect_repo` extracts `owner/repo` from SSH (`git@github.com:owner/repo.git`), HTTPS (`https://github.com/owner/repo.git`), and `http://github.com/...` remote URLs; the trailing `.git` is optional
- `resolve_repo` prefers explicit config repo over auto-detected repo; returns error if neither is available
- `gh_is_available` returns true only when `gh auth status` succeeds (CLI is installed and authenticated)
- `fetch_issue`, `list_issues`, and issue verification use in-process GitHub REST and never launch a `gh` provider process
- Every issue read/list/verify path requires `GITHUB_TOKEN`; authenticated `gh` state is not a fallback
- `fetch_issue_api` uses a 10-second HTTP timeout; returns error on network failure
- Issue state is normalized to lowercase (`"open"` / `"closed"`) regardless of API response format
- `verify_spec_issues` classifies each issue as valid (open), closed, not_found, or error with detailed messages
- Issue verification preflights repository access once and revalidates it after apparent absence;
  inaccessible repositories, authentication, transport, timeout, and malformed-provider failures
  are errors rather than not_found.
- Batch verification globally deduplicates at most 100 issue IDs and bounds REST request duration
  plus repository-preflight and total elapsed time.
- `create_drift_issue` is the only issue operation that invokes `gh`; no REST write fallback is provided
- `create_drift_issue` creates issue titled "Spec drift detected: {path}" with formatted error list in body
- Drift issues are created with configurable labels (default `["spec-drift"]`, set via `github.drift_labels`)
- `list_issues` lists every open issue through in-process REST, requires `GITHUB_TOKEN`, skips pull
  requests, rejects any provider page above 100 entries before item parsing, follows strict GitHub
  `Link` pagination for at most 100 pages, and fails on malformed links, duplicate issue numbers,
  or a page-limit truncation. Pull-request entries count toward the provider-page bound. Before PR
  filtering, every raw item must have a valid marker shape, positive identity, nonempty title,
  nonempty names for any labels, exact open state, and canonical repository/resource/number URL
  identity; the number segment must exactly equal canonical decimal `u64` spelling, so leading
  zeros are rejected. Duplicate raw
  identities within or across pages fail even when pull requests would otherwise be filtered. Each
  next link must retain the requested repository issues endpoint and exact `state=open`,
  `per_page=100`, label, and page query semantics.
- Auth tokens (`GITHUB_TOKEN`) are redacted from REST request error messages via `redact_token` before being surfaced (defense-in-depth)

### REQ-github-002

The maintained GitHub Action SHALL expose an immutable exact-version ref and a verified floating
major compatibility ref whose default binary version is synchronized only after exact-version
artifacts pass supported-platform verification.

Acceptance Criteria

- The composite Action's current default matches the promoted stable package version.
- An immutable `v<major>.<minor>.<patch>` Action ref resolves to the integrated release commit.
- The floating `v<major>` ref resolves to that same commit only after pinned consumers pass on
  Linux, macOS, and Windows.
- Documentation distinguishes immutable pinning from the floating compatibility ref.
- A failed exact-version asset or Action smoke test leaves the floating ref unchanged.

### REQ-github-003

Hosted JavaScript verification SHALL select an exact supported Bun runtime rather than resolving
the newest Bun tag during each workflow run.

Acceptance Criteria

- Site deployment, site CI, and VS Code extension CI use the same exact Bun version.
- Setup does not query the Bun repository's live tag-discovery API to select a runtime version.
- The pinned runtime successfully installs frozen dependencies and passes the maintained site and
  extension verification commands.

### REQ-github-004

The maintained GitHub Action SHALL promote the 5.2.0 release through an immutable exact-version
ref whose default binary version synchronizes only after exact-version artifacts pass
supported-platform verification, with the floating major ref following the same contract.

Acceptance Criteria

- The composite Action's default and maintained consumer pins read exactly 5.2.0 once the
  accepted release commit lands on main.
- The immutable `v5.2.0` Action ref resolves to the integrated release commit after publication.
- The floating `v5` ref moves to 5.2.0 only after pinned consumers pass on Linux, macOS, and
  Windows.
- A failed exact-version asset or Action smoke test leaves the floating ref and prior default
  unchanged.
