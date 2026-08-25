---
spec: github.spec.md
---

## Post-5.0 Test Debt

- [ ] Live REST success paths and explicit `gh` issue creation remain integration-only;
  deterministic REST failure, timeout, malformed-response, deduplication, and cap paths are unit-tested

## Done

- [x] Batch issue verification with injected deterministic orchestration, global deduplication/caps, and per-spec attribution
- [x] Bound in-process REST operations and the complete repository-preflight/fetch batch
- [x] Revalidate repository access after apparent issue absence before classifying not-found

- [x] Pin one exact Bun runtime and setup Action ref across Pages, site CI, and VS Code extension CI with a deterministic drift guard that rejects duplicate or unexpected setup steps
- [x] Add dependency-free deterministic 5.1.1 release-version consistency validation across current distribution surfaces, parsing every README/site YAML example so named/nested Action steps and block/flow `with.version` inputs reject stale or moving pins on Python 3.10+
- [x] Close late release-validator bypasses by normalizing Action repository names case-insensitively, scanning indented, metadata-bearing, backtick, or tilde YAML fences, and recognizing quoted or flow-style workflow `uses` keys while preserving exact ref checks
- [x] Parse maintained workflow steps structurally so every YAML-equivalent action/input mapping is validated, scan current root security guidance, and keep optional PR decoration fork-safe
- [x] `detect_repo` / `parse_repo_from_url` — auto-detect `owner/repo` from SSH and HTTP(S) git remotes
- [x] `resolve_repo` — explicit config repo wins over auto-detection, error when neither is available
- [x] `GITHUB_TOKEN`-required in-process REST reads/listing/verification, with `gh_is_available` reserved for issue creation
- [x] `fetch_issue` / `verify_spec_issues` — classify `implements`/`tracks` references as valid/closed/not_found/error
- [x] `list_issues` via encoded REST parameters, strict fields/links, PR filtering, bounded full pagination, and no silent duplicates/truncation
- [x] Bind pagination links to the requested repository issues endpoint and exact query semantics
- [x] Reject issue-list provider pages above 100 entries before parsing items, including pull-request entries
- [x] Validate every raw issue/pull-request item before filtering, including open-only state and
  exact repository/resource/number URL identity.
- [x] Require provider item URL numbers to use exact canonical decimal spelling, rejecting leading
  zeros even when they parse to the same numeric issue or pull-request ID.
- [x] Reject duplicate raw identities within and across pages even when pull requests are filtered.
- [x] Reject pull-request markers returned by the direct issue-detail endpoint.
- [x] Forbid `gh` construction in every read module on all platforms and prove Unix token-present
  read/list/verify/import paths never execute a PATH sentinel
- [x] `create_drift_issue` — `gh`-only issue creation titled "Spec drift detected: {path}"
- [x] Token redaction in REST error messages via `redact_token` (4.3.5)
- [x] Fail closed on inaccessible repositories, bound/deduplicate issue verification, and disable legacy `gh` read providers (CHG-0063)
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Open

- [x] Confirm focused evidence that `pull_request: null` and every non-object marker reject the
  complete provider page before PR filtering.
- [ ] After publishing each exact 5.x release, smoke-test the immutable Action ref on Linux,
  macOS, and Windows before creating or advancing the floating `v5` ref

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
- [x] Prepare the 5.2.0 release: synchronized version surfaces and the Action promotion contract (REQ-github-004)
- [x] Bind v2 finalization to execution/review verdict evidence and preserve v1/fork-safe CI routes
- [x] Authenticate review check provenance and append-only attempts, share CI limits, and preserve
  exact archive-tree binding across squash/rebase integration
- [x] Finish binding Ubuntu/macOS/Windows release qualification and publication to one immutable
  annotated RC tag while making Ubuntu the ordinary-PR integration authority (CHG-0075). The two
  immutability rulesets are provisioned and live-proven: `SpecSync immutable RC tags` (21432132)
  and `SpecSync immutable final tags` (21432148), both active with no bypass actor.
- [x] Decide the fate of App-only final-tag creation: **no GitHub App.** The `SpecSync final tag
  creation` ruleset, the release App, and the protected `release` environment are not being
  provisioned, so the App plumbing was retired instead of left failing closed. `promote` now
  creates the final tag with the workflow's own `GITHUB_TOKEN` under a `contents: write` permission
  scoped to that one job, and names no deployment environment. The cost is recorded at the job, in
  `docs/ci-confidence.md`, and as a warning annotation on every run: anyone able to run
  `release.yml` from the default branch can cause `vX.Y.Z` to be created, and no reviewer stands in
  between. Tag immutability is unchanged.
- [ ] Optional hardening, if release authority should ever be narrower than workflow-run access:
  create a `release` deployment environment with required reviewers and a `main`-only deployment
  branch policy, re-add `environment: release` to `promote`, and restore a qualification check that
  proves those rules are still in place. Do not re-add the reference before the environment exists
  with real protection rules — an auto-created environment has none.
- [x] Reuse authenticated required checks across bounded review/archive metadata descendants without
  crossing code edges or allowing cancelled republications to poison exact-SHA success (CHG-0077)
