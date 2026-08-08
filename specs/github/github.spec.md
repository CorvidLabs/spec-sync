---
module: github
version: 25
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

## Invariants

Every path that can be merged can reach the required CI gate; a path the CI
workflow cannot trigger can never report the gate and blocks its pull request.

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

### Scenario: Same-PR archive child

- **Given** an implementation parent has green required CI and scoped review
- **When** its only child moves the matching active package into the dated archive with valid finalization evidence
- **Then** required CI runs the lightweight archive-integrity lane without repeating product tests or scoped review

### Scenario: Trust validates identity without repeating CI tests

- **Given** GitHub CI owns formatting, linting, product tests, strict spec coverage, and audit
- **When** the Trust lifecycle runs for the same product tip
- **Then** it checks the release binary, contract, risk, and provenance without re-running the full
  local `verify` lane or `cargo test`

### Scenario: Promote an immutable release candidate

- **Given** annotated tag `v6.0.0-rc.1` resolves to one merged commit with valid archive binding
- **When** the same `release-candidate` Fledge lane succeeds on Ubuntu, macOS, and Windows for that SHA
- **Then** promotion may create `v6.0.0` at that SHA and publish artifacts; any missing, failed, stale,
  mixed-SHA, or replaced-marker evidence fails closed

### Scenario: Finalize after review metadata

- **Given** one product tip has green CI and a later child contains only valid scoped-review metadata
- **When** a same-PR archive child is finalized without waiting for duplicate product checks
- **Then** the archive gate reuses the product tip's authenticated checks and refuses to cross any
  intervening product-code edge

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
| Direct issue-detail response contains a `pull_request` marker | Returns a provider error; verification/import does not treat the pull request as an issue |
| Issue-list entry has `pull_request: null` or another non-object marker | Entire listing fails as malformed provider data; null cannot masquerade as an ordinary issue |
| Raw issue or pull-request item is closed, has malformed fields, or has mismatched repository/resource/number URL identity | Entire listing fails before pull-request filtering |
| Duplicate raw item identity within or across list pages, including pull requests | Entire listing fails; filtered pull requests cannot hide duplicates |
| Issue-list page contains 101 or more provider entries | Entire listing fails before parsing any entry, even if overflow entries are pull requests or malformed |
| Issue listing still has a next page after 100 pages | Entire listing fails instead of truncating |
| Archive child changes code/spec/tests or rewrites immutable package evidence | Archive-only validation fails; it never skips the product matrix on an unproven diff |
| Release commit lacks a successful merge-bound archive check | Release validation fails before building artifacts |
| RC marker is lightweight, malformed, moved, or has conflicting workflow history | Qualification and promotion fail; a fresh annotated RC marker is required |
| Platform evidence is missing, unsuccessful, or bound to another tag/SHA | Final tag creation and release upload are refused |
| Immutable RC/final tag rulesets are absent, inactive, incomplete, or grant a forbidden bypass | RC qualification and promotion fail before final-tag creation |
| Metadata ancestry contains code, a merge-only side parent, exceeds 32 first parents, or has foreign/malformed/unsuccessful evidence | Reuse stops and the archive/Trust gate fails closed instead of borrowing older checks |

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
| 2026-07-22 | CHG-0063 independent-review follow-up: Bound raw issue-list pages and reject null or non-object pull-request markers before filtering |
| 2026-07-22 | CHG-0063 final adversarial follow-up: Validate every raw issue/pull-request item as open with exact URL identity and canonical decimal number spelling, and reject raw duplicates before PR filtering |
| 2026-07-22 | CHG-0063 final agent review: reject pull-request payloads from direct issue-detail reads |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-30 | CHG-0068: Bind archive digests to v2 execution/review evidence, preserve v1 full validation, and make scoped review fork-safe |
| 2026-07-30 | CHG-0068: Share native/hosted review bounds, authenticate review check provenance and append-only attempts, and preserve v2 archive validation across squash/rebase integration |
| 2026-07-30 | CHG-0068: Make merged-fork archive publication base-controlled, bound archive reconstruction, and protect the complete workflow/local-Action check-production surface |
| 2026-07-30 | CHG-0068 review hardening: Pin privileged archive checkout and make trust-policy protection descendant- and rename-safe |
| 2026-07-30 | CHG-0068 adversarial hardening: Protect root Action manifests from optimized trust reuse |
| 2026-07-30 | CHG-0068 adversarial hardening: Preserve NUL filename boundaries in trusted-policy matching |
| 2026-07-30 | CHG-0068 review hardening: Reject archive rewrite-then-restore history |
| 2026-08-01 | CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi: Simplify SpecSync CI to one expensive-suite authority with residual Trust identity gates, preserving full local verification and documenting the 95% confidence model |
| 2026-08-01 | CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida: Bind cross-platform release qualification, final tagging, and publication to one immutable RC commit while making Ubuntu the ordinary-PR integration authority |
| 2026-08-01 | CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida: Bind release-candidate validation and final publication to one immutable candidate SHA across Ubuntu macOS and Windows |
| 2026-08-02 | CHG-0077: Reuse authenticated product-tip CI across bounded metadata-only descendants and prevent cancelled republications from poisoning an earlier exact-SHA success |
| 2026-08-02 | CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre: Reuse successful exact-PR CI provenance across metadata-only descendants and prevent later cancelled or failed republishing from poisoning an earlier successful exact-SHA result |
| 2026-08-02 | CHG-0077 review hardening: Traverse parent-bound workflow-v2 archives, bind reusable job checks to exact jobs, and ignore unsuccessful policy republications without rejecting GitHub-rewritten check URLs |
| 2026-08-02 | CHG-0077 late review hardening: Match native manifest spelling, empty-manifest, and unsigned timestamp semantics without tracking interpreter bytecode |
| 2026-08-02 | CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change: Delete the CI reimplementation of the SDD lifecycle and rely on specsync change audit, removing the separate-archive-tip constraint |
| 2026-08-04 | CHG-0082-let-documentation-only-pull-requests-reach-the-required-ci-gate: Let documentation-only pull requests reach the required CI gate |
| 2026-08-06 | CHG-0087-decide-pull-requests-in-seconds-instead-of-minutes: Decide pull requests in seconds instead of minutes |
| 2026-08-08 | CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha: Ship-status live GitHub check-run trust for product parent SHA |
