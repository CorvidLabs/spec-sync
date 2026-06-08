---
spec: github.spec.md
---

## Done

- [x] `detect_repo` / `parse_repo_from_url` — auto-detect `owner/repo` from SSH and HTTP(S) git remotes
- [x] `resolve_repo` — explicit config repo wins over auto-detection, error when neither is available
- [x] `gh_is_available` gate, with `gh` CLI preferred and `GITHUB_TOKEN` REST fallback
- [x] `fetch_issue` / `verify_spec_issues` — classify `implements`/`tracks` references as valid/closed/not_found/error
- [x] `list_issues` (gh + REST, PR filtering on the REST path)
- [x] `create_drift_issue` — `gh`-only issue creation titled "Spec drift detected: {path}"
- [x] Token redaction in REST error messages via `redact_token` (4.3.5)
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Open

- [ ] No automated coverage for the REST/`gh` network paths (only URL parsing is unit-tested); these remain manual/integration-only

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
