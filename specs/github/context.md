---
spec: github.spec.md
---

## Key Decisions

- **In-process REST reads**: `fetch_issue`, `list_issues`, and batch verification require an explicit `GITHUB_TOKEN` and use direct `ureq` requests. They do not launch `gh`; issue creation (`create_drift_issue`) is the only `gh` path.
- **Bounded complete listing**: `list_issues` uses 10-second page requests and strict `Link`
  pagination for at most 100 pages. Each raw provider page is capped at 100 entries before item
  parsing, and pull-request entries count toward that bound. Malformed links, oversized pages,
  duplicate issue IDs, or a continuing next page at the cap are errors rather than silently
  truncated imports. A next link is accepted only when it preserves the requested repository
  issues endpoint and exact open-state, page-size, label, and page query semantics.
- **Strict PR marker**: an issue-list entry is a pull request only when `pull_request` is an object.
  The marker may be absent for an issue, but explicit `null` and every other type are malformed
  provider data and reject the complete page before filtering.
- **Validate raw items before filtering**: every raw issue or pull-request item must be open and
  carry a positive number, nonempty title, nonempty names for any labels, and an exact github.com URL whose repository,
  resource (`issues` versus `pull`), and canonical decimal number spelling match the item. Leading
  zeros are rejected even when they parse to the same numeric ID. Raw identities are checked for
  duplicates within and across pages before pull requests are removed, so filtering cannot hide a
  malformed, closed, URL-confused, or duplicate provider item.
- **Token redaction**: `redact_token` strips any verbatim `GITHUB_TOKEN` occurrence from REST error strings before they surface (added 4.3.5). The token travels in the `Authorization` header, so this is defense-in-depth against a misbehaving proxy/redirect echoing it back.
- **State normalization**: verified issue `state` is lowercased (`"open"`/`"closed"`) so callers compare REST results consistently.
- **Fail-closed batch verification**: issue checks preflight repository access once, globally
  deduplicate at most 100 IDs across the batch, and revalidate access after typed not-found
  outcomes. REST operations and the complete batch are deadline-bounded; ambiguous
  private-repository 404s remain inconclusive.
- **github.com only**: URL parsing handles `git@github.com:`, `https://github.com/`, and `http://github.com/`; GitHub Enterprise hosts are out of scope.
- **Deterministic hosted Bun runtime**: Pages, site CI, and VS Code extension CI use one exact Bun
  version and the expected `setup-bun` Action ref. `.github/scripts/validate-workflow-runtime-pins.py`
  validates every matching setup step and rejects moving refs, duplicates, unexpected jobs, or a
  missing nested `bun-version`, preventing a live tag-discovery dependency from returning. Action
  repository names are matched case-insensitively, as GitHub resolves them, while refs remain exact
  and case-sensitive so mixed-case owner/repository spellings cannot bypass the pin guard.
- **Monotonic Action promotion**: immutable `v<major>.<minor>.<patch>` refs are verified before the
  compatible floating `v<major>` ref advances. Release metadata remains synchronized through
  `.github/scripts/validate-release-version.py`, which rejects every README/site Action ref other
  than the exact candidate ref, including moving branch names such as `main`. It parses fenced YAML
  through Psych, including case-insensitive backtick or tilde fences with up to three leading spaces,
  optional horizontal space before the language, longer valid closing fences, unclosed fences that
  extend to end-of-document, and metadata such as `title="ci.yml"`, so named/nested `uses` steps and
  block or flow `with.version` mappings
  are covered without mistaking cross-project reference prose for an Action step. Workflow
  validation structurally parses block and flow mappings, including quoted keys, arbitrary valid
  list-marker spacing, and `uses` keys after other step fields. Action repository names are normalized case-insensitively,
  while the selected release ref is compared exactly. Root `SECURITY.md` changes trigger the same
  CI validation path as other maintained Action documentation, and its inline Action recommendation
  must match the current floating major ref. The release workflow is part of both the maintained
  workflow scan and the CI path triggers. The packaged Action consumer and Trust gate must retain
  their exact runner-local candidate mirrors so they cannot silently test an already-published binary.
- **Hermetic release guards**: release and runtime-pin validators require no Python site packages;
  the release guard uses Ruby's standard-library Psych parser for full YAML syntax validation.
  Cargo metadata is read without Python 3.11-only `tomllib`, lifecycle verification declares the
  Ruby preflight explicitly, and hosted CI provisions a pinned Ruby runtime, so Python 3.10+
  verification depends on neither ambient PyYAML nor an undeclared hosted runtime.
- **Fork-safe scoped review**: qualifying fork pull requests run the same read-only analysis with no
  repository secrets; only PR comments/review writes are disabled when the token cannot decorate
  the source PR.
- **Lifecycle routing**: workflow-v2 same-PR archives use the lightweight execution-bound lane;
  workflow-v1 archives stay on the historical full matrix, and parent-state authentication prevents
  a v2 archive from downgrading itself. Review reuse requires schema 2, a passing verdict, an
  independent reviewer, contract/execution/workspace digest parity, and bounded every-parent
  freshness from the reviewed implementation commit. Hosted finalization caps Git command time,
  output bytes, descendant count, and parent count from the same committed limits document used by
  native validation; exceeding any bound fails closed. Review evidence carries the required check
  name and append-only attempts, while the finalizer authenticates the official Actions app, exact
  PR/run/head, and check success. Post-merge validation compares exact archive subtree identity, so
  squash/rebase integration remains verifiable after discarded implementation objects disappear.
- **Base-trusted policy boundary**: optimized lifecycle reuse is guarded by a SHA-pinned
  `pull_request_target` workflow with read-only inspection and a separate check publisher. It fetches
  the exact candidate as Git objects without checking out or executing candidate content, blocks
  changes to every workflow/local-Action definition—including root `action.yml`/`action.yaml`—plus
  lifecycle classifiers/limits and the workflow-v2 baseline. The path scan disables rename
  detection and matches local-Action descendants, so add, modify, delete, and move operations all
  expose protected paths. It preserves Git's NUL filename boundaries and full-matches raw bytes, so
  embedded newlines cannot split one protected workflow path into unprotected text lines. It
  publishes an external ID bound to the
  trusted workflow revision and PR head. Review reuse, finalization, and post-merge binding verify
  the exact event, repository, PR, head, workflow path, and revision. The initial guard introduction
  is frozen to CorvidLabs/spec-sync PR #480, its exact base and branch identity, and a required set
  of newly added policy files; it still requires full CI and independent review. Later policy edits
  require a newly pinned GitHub required-workflow revision and cannot use the optimized path.
- **Fork-safe archive publication**: merged archive publication runs from the immutable base
  workflow on `pull_request_target: closed`; PR head and merge identities are fetched only as Git
  objects, never checked out or executed, and the checkout Action itself is pinned to a full commit
  SHA. Archive-introduction and release reconstruction share a
  protected verifier that caps commit history, parents, time, and streamed output and rejects
  post-introduction archive rewrites. It inspects every bounded archive-path-touching commit and
  readable parent, so rewrite→restore history is not reduced to a clean final tree.

## Key Files

- `src/github.rs` - Main implementation: repo detection, in-process REST reads/listing/verification, `gh`-only `create_drift_issue`, and `redact_token`
- `src/commands/mod.rs` - `create_drift_issues` wires `github.drift_labels` (default `["spec-drift"]`) into `create_drift_issue`
- `specs/github/github.spec.md` - Module specification
- `specs/github/requirements.md` - User stories and acceptance criteria
- `.github/scripts/validate-release-version.py` - Current package, Action, docs, CI consumer, and
  Trust candidate version consistency
- `.github/scripts/validate-workflow-runtime-pins.py` - Exact hosted Bun runtime enforcement

## Current Status

CHG-0063 verification is active. URL parsing, endpoint-bound and provider-page-bounded pagination,
REST provider classification/revalidation, malformed responses, global deduplication/caps, complete
deadlines, transport failures, and rejection of legacy `gh` reads without process spawning have
focused source regressions. Raw-page coverage validates every issue and pull-request item before
filtering, including open-only state, exact URL identity, object-only markers, and duplicate raw
identities within/across pages; exact URL identity includes canonical decimal number spelling.
Direct issue-detail responses are also issue-only: any pull-request marker is rejected before an
import can consume the payload.
Live network paths remain integration-only. The 5.1.1 release
candidate adds deterministic Action/runtime distribution checks, while
external exact/floating ref smoke tests remain publication-time gates.
The 5.2.0 release promotion follows REQ-github-004: Action default and consumer pins move to the exact version through the accepted release change, and the floating v5 ref advances only after exact-version artifacts pass Linux/macOS/Windows verification.
