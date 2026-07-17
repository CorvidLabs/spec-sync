---
spec: github.spec.md
---

## Post-5.0 Test Debt

- [ ] No automated coverage for the REST/`gh` network paths (only URL parsing is unit-tested); these remain manual/integration-only

## Done

- [x] Pin one exact Bun runtime and setup Action ref across Pages, site CI, and VS Code extension CI with a deterministic drift guard that rejects duplicate or unexpected setup steps
- [x] Add dependency-free deterministic 5.1.1 release-version consistency validation across current distribution surfaces, parsing every README/site YAML example so named/nested Action steps and block/flow `with.version` inputs reject stale or moving pins on Python 3.10+
- [x] Close late release-validator bypasses by normalizing Action repository names case-insensitively, scanning indented or metadata-bearing YAML fences, and recognizing quoted workflow `uses` keys while preserving exact ref checks
- [x] `detect_repo` / `parse_repo_from_url` — auto-detect `owner/repo` from SSH and HTTP(S) git remotes
- [x] `resolve_repo` — explicit config repo wins over auto-detection, error when neither is available
- [x] `gh_is_available` gate, with `gh` CLI preferred and `GITHUB_TOKEN` REST fallback
- [x] `fetch_issue` / `verify_spec_issues` — classify `implements`/`tracks` references as valid/closed/not_found/error
- [x] `list_issues` (gh + REST, PR filtering on the REST path)
- [x] `create_drift_issue` — `gh`-only issue creation titled "Spec drift detected: {path}"
- [x] Token redaction in REST error messages via `redact_token` (4.3.5)
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Open

- [ ] After publishing each exact 5.x release, smoke-test the immutable Action ref on Linux,
  macOS, and Windows before creating or advancing the floating `v5` ref

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
