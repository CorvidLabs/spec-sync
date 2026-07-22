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
- [x] Forbid `gh` construction in every read module on all platforms and prove Unix token-present
  read/list/verify/import paths never execute a PATH sentinel
- [x] `create_drift_issue` — `gh`-only issue creation titled "Spec drift detected: {path}"
- [x] Token redaction in REST error messages via `redact_token` (4.3.5)
- [x] Fail closed on inaccessible repositories, bound/deduplicate issue verification, and disable legacy `gh` read providers (CHG-0063)
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Open

- [ ] After publishing each exact 5.x release, smoke-test the immutable Action ref on Linux,
  macOS, and Windows before creating or advancing the floating `v5` ref

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
- [x] Prepare the 5.2.0 release: synchronized version surfaces and the Action promotion contract (REQ-github-004)
