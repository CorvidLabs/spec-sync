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
- Direct issue-detail reads reject payloads carrying a `pull_request` marker before verification
  or import conversion.

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
- Direct issue-detail responses reject any `pull_request` marker before returning importer data.
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

### REQ-github-005

Maintained GitHub automation SHALL finalize and validate change archives on the originating PR
without repeating the implementation matrix or bypassing merge protections.

Acceptance Criteria

- Required implementation checks and one schema-v2 passing scoped review for agent-authored work
  bind the implementation parent commit, execution digest, workspace digest, append-only review
  trail, and required GitHub Actions check provenance.
- The required merge gate remains incomplete until same-PR finalization; a review-metadata-only
  child reuses its parent's product checks and independent review without rerunning either.
- Same-PR finalization produces a child commit containing only exact approved lifecycle/archive
  changes.
- The archive-only lane verifies parent green checks, exact diff classification, unchanged delivery
  tree, archive integrity, bidirectional ownership, and finalization digest, then reports to required
  CI without selecting the full product matrix or scoped reviewer again.
- GitHub branch protection or merge queue performs the merge; SpecSync automation never invokes a
  merge API.
- A lightweight post-merge job may bind actual merge SHA/tree to the archive digest and retry
  transient failures without writing code files.
- Squash/rebase integration preserves an exact archive-subtree anchor so fresh clones can validate
  workflow-v2 evidence after discarded implementation commits become unreachable.
- Every bounded archive-path-touching commit and readable parent retains the exact introduction
  subtree, so an intermediate deletion or rewrite cannot be concealed by restoring final bytes.
- Release validation rejects integrated changes lacking valid same-PR finalization and merge binding.
- Workflow permissions are least privilege and fork-controlled input cannot forge parent status,
  review, allowlisted paths, or archive identity.
- Merged-fork archive publication executes only immutable base-controlled workflow code and fetches
  PR identities as Git objects without checking out or executing candidate content; every
  privileged executable Action dependency is pinned to a full commit SHA.
- The trusted policy guard rejects changes to every `.github/workflows/*.yml`,
  `.github/workflows/*.yaml`, root `action.yml`/`action.yaml`, `.github/actions/**` definition, and
  the workflow-v2 baseline. It disables rename detection so protected deletions and moves remain
  visible, preserves NUL filename boundaries, and full-matches each raw Git path independently; its
  initial exception is frozen to one repository, PR, exact base, branch identity, canonical
  exact-base baseline, and required added-file set.
- Workflow-v1 archives select the historical full-validation path; a v2 parent cannot downgrade
  itself to that route.
- Fork PRs run the same read-only scoped-review analysis without secrets, comments, or review writes.
- Classifier and finalizer history limits come from one committed limits document shared with the
  native validator.

### REQ-github-006

Hosted verification SHALL assign each expensive confidence signal to one authoritative workflow,
while Trust SHALL retain release-binary identity, strict contract, risk, and provenance checks
without re-running the full product test suite.

Acceptance Criteria

- GitHub CI remains the authority for formatting, linting, full Rust tests, strict spec coverage,
  audit, coverage measurement, site, editor extension, and packaged-action consumer checks.
- `.trust.toml` invokes a dedicated `trust-lifecycle` lane that does not contain `cargo test`,
  clippy, or the full `verify` lane.
- `lanes.verify` remains the full local completion suite for agents and humans.
- Documentation identifies the current multi-OS matrix as Tier B work that requires a separately
  pinned protected-workflow update; this change does not silently weaken platform coverage.
- The thin PR contains no protected workflow files and no ship-status/product-code feature.

### REQ-github-007

Release qualification SHALL bind Ubuntu, macOS, and Windows results and final publication to one
immutable release-candidate commit, while ordinary product pull requests SHALL use Ubuntu as the
authoritative integration platform.

Acceptance Criteria

- Ordinary development/product PRs do not schedule macOS or Windows integration jobs.
- An RC branch is frozen by an immutable annotated `vX.Y.Z-rc.N` marker resolving to one full SHA.
- Active tag rulesets let humans create new RC markers but forbid their update/deletion, allow only
  a dedicated release GitHub App to create final tags, and forbid every actor from updating or
  deleting final tags. Its private key is available only to the protected `release` environment's
  promotion job, which mints a short-lived token scoped to the repository.
- Every required platform runs the same named Fledge RC lane at that exact SHA.
- Changing candidate content requires a new RC marker and fresh platform evidence.
- Promotion fails closed unless Ubuntu, macOS, and Windows are green for the unchanged candidate SHA.
- The final `vX.Y.Z` tag is created only after promotion succeeds and points to that same SHA.
- Release uploads independently reject mismatched marker, tag, checkout, evidence, or artifact SHA.
- Release-chain Actions and executables have independent immutable pins, and publication freshly
  revalidates tags, actual checkout, original platform evidence, and package hashes.

### REQ-github-008

Review-only and archive-only descendants SHALL reuse successful CI provenance from the nearest
eligible first-parent product ancestor without allowing later unsuccessful republication or unrelated
GitHub evidence to authorize the change.

Acceptance Criteria

- Reuse walks at most 32 first parents. The current child must classify as exact review/archive
  metadata, and traversal stops before any earlier child that is not exactly the same change's
  `review.json` plus `review-attempts.json` update.
- Metadata-child check republications are not treated as fresh product evidence, and a product
  boundary with no eligible success cannot borrow an older product commit's checks.
- The provenance helper and its focused test cannot change without also changing the separately
  protected required-CI workflow.
- Implementation-ready, scoped-review, and Trust evidence share one product ancestor; the two CI
  checks share one workflow run.
- Every reusable check is successful, exact-SHA-bound, GitHub-Actions-authored, bound to the same
  pull request and repository, and produced by the expected workflow.
- A newer cancelled or failed trusted-policy publication does not override an earlier authenticated
  success for the same exact SHA.
- Missing, foreign, stale, wrong-workflow, second-parent, malformed, unsuccessful-only, over-limit,
  or ambiguous evidence fails closed.
- Eligible metadata descendants do not rerun the full product matrix.
