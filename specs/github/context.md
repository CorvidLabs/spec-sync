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
- **Lifecycle authority lives in the product, not in CI — and the CI copy is GONE**: `specsync
  change audit --strict` is the single authority on lifecycle coherence. Everything this section
  used to describe — lifecycle routing between a lightweight v2 lane and a v1 full matrix, a
  SHA-pinned `pull_request_target` base-trusted policy guard frozen to PR #480, post-merge archive
  publication on `pull_request: closed`, and metadata-descendant provenance reuse — was **deleted
  by PR #499** (802ca13b), which removed ~7,257 lines: `.github/workflows/finalize-change.yml`,
  `lifecycle-policy-guard.yml`, `post-merge-archive.yml`, and the
  `reuse-check-from-ancestors.py` / `verify-trusted-policy-check.py` /
  `verify-archive-introduction.py` helpers with their harnesses. There is **no
  `pull_request_target` workflow in this repository** and no finalization or post-merge workflow;
  the surviving five are `ci.yml`, `pages.yml`, `rc-assets.yml`, `release.yml`, and `trust.yml`.
  Protected-path authorization is `.github/CODEOWNERS`. `.github/scripts/lifecycle-validation-limits.json`
  is retained only because `src/change.rs` reads it. Two of the live defects #499 removed were in
  the CI copy and not in SpecSync, which is the argument for not rebuilding it: a reimplementation
  of a shipped rule drifts from it, and only the copy is unshipped and untested by users.

## Key Files

- `src/github.rs` - Main implementation: repo detection, in-process REST reads/listing/verification, `gh`-only `create_drift_issue`, and `redact_token`
- `src/commands/mod.rs` - `create_drift_issues` wires `github.drift_labels` (default `["spec-drift"]`) into `create_drift_issue`
- `specs/github/github.spec.md` - Module specification
- `specs/github/requirements.md` - User stories and acceptance criteria
- `.github/scripts/validate-release-version.py` - Current package, Action, docs, CI consumer, and
  Trust candidate version consistency
- `.github/scripts/validate-workflow-runtime-pins.py` - Exact hosted Bun runtime enforcement
- `fledge.toml` and `.trust.toml` - Keep full local verification separate from hosted Trust's
  residual lifecycle prerequisite
- `docs/ci-confidence.md` - CI/Trust ownership, confidence tiers, and protected Tier B follow-up
- `.github/workflows/release.yml` and `.github/scripts/validate-release-candidate.py` - Resolve an
  annotated RC marker, qualify its exact SHA through one Fledge lane on three platforms, and refuse
  final tagging/publication when evidence identities diverge
- `specsync change audit --strict` in CI - SpecSync is the single authority on lifecycle coherence.
  CI does not reimplement lifecycle rules against commit topology

## Current Status

CHG-0063 is archived and its work is shipped; `.specsync/changes/` holds no active workspace for
it. URL parsing, endpoint-bound and provider-page-bounded pagination,
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
The 5.2.0 release promotion follows REQ-github-004: Action default and consumer pins move to the exact version through the accepted release change, and the floating major ref advances only after exact-version artifacts pass Linux/macOS verification. As of 6.0 no Windows binary is published, so a Windows consumer is refused by the Action rather than smoke-tested; the release-candidate qualification lane still runs on Windows.

CHG-0075 moves routine integration authority to Ubuntu and reserves macOS/Windows spend for one
immutable release candidate. The RC tag—not the movable staging branch—is the release identity.
Qualification records tag, SHA, platform, lane, and outcome; promotion re-resolves the marker and
accepts only an official successful release-workflow check for the same unchanged SHA. Release
archive provenance must match the actual post-merge `pull_request` workflow event. Matching release
tags use **two** active policies, both live and both verified by `resolve`: humans may create RC
markers and final tags, and no actor may update or delete either. `SpecSync immutable RC tags`
(21432132) covers `refs/tags/v*.*.*-rc.*`; `SpecSync immutable final tags` (21432148) covers
`refs/tags/v*.*.*` excluding the RC pattern. Neither grants bypass to anyone, and the validator
rejects any broadening. The upload job then re-resolves tags and actual checkout after builds and
revalidates original platform evidence plus package hashes.

CHG-0075 originally specified a third policy — `SpecSync final tag creation`, admitting only the
dedicated CorvidLabs release GitHub App — plus a protected `release` environment holding the App's
private key. Neither was ever provisioned: no App exists, its id variable and private-key secret
were never set, and there is no `release` environment. Demanding them failed `release.yml` on every
RC tag from `v6.0.0-rc.1` through `rc.6`, so the two rulesets that do exist were never reached and
the check never once passed. Qualification now requires only those two.

The owner then decided against a release App entirely, so the App is gone from the workflow rather
than pending. `promote` creates the final tag with the workflow's own `GITHUB_TOKEN` under a
`contents: write` permission scoped to that single job; the workflow-level default stays read-only.
The `environment: release` reference was removed with it, because GitHub materializes a referenced
environment on first use with no protection rules, so naming an environment this repository does
not have would publish a deployment gate that gates nothing. Making promotion a real gate means
creating the environment with reviewers and a `main`-only branch policy first, then re-adding the
reference and a check that proves it.

What that costs is stated on **every run**, green ones included, as `::warning::` annotations and
in the step summary, and repeated at the `promote` job itself: final-tag creation is unrestricted;
the tag is minted by this workflow's own token rather than a separate identity, so anyone able to
run `release.yml` from the default branch can cause `refs/tags/vX.Y.Z` to be created; and no
deployment-environment approval stands in between. Tag immutability is untouched — both rulesets
still admit no bypass actor, so no one, including that token, can move or delete a tag once it
exists. Nothing in this repository triggers on a final-tag push (`release.yml` is the only `tags:`
trigger and matches RC tags only), so using `GITHUB_TOKEN` breaks no downstream automation; work
that must react to `vX.Y.Z` has to be called from inside `release.yml`.

CHG-0077 repairs the finalization tip dance reproduced by PR #492: a review child no longer orphans
the green product tip, and a later cancelled/failed policy republication no longer poisons an earlier
authenticated success for the same SHA. The repair does not add a command, lifecycle state, approval,
or alternate archive path.

PR #494 review exposed three same-contract gaps before merge. Historical traversal recognized only
review pairs, so separately finalized workflow-v2 changes formed a false product boundary. Generic
check reuse also accepted run-level URLs without proving an exact job/check binding, and policy-run
selection treated failed republications as ambiguity. The invariant is now explicit: historical
archive edges carry exact parent commit/tree finalization proof; reusable job evidence names and
authenticates one exact successful job; and GitHub-rewritten policy check URLs retain CHG-0076
compatibility while only successful matching policy runs participate in canonical-URL disambiguation.
Late review then found three native-parity edges plus a generated-artifact leak.

Read both of those paragraphs as history. The Python verifier and the traversal helper they
describe were deleted by PR #499 along with the rest of the CI lifecycle copy, so the "the verifier
now …" clauses point at files that no longer exist. What survives is native: `src/change.rs`
rejects the unsupported `non-file` archive-entry spelling and out-of-range lifecycle timestamps,
and accepts a zero-entry acceptance manifest.
