---
spec: cmd_issues.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add coverage for the no-references path (specs without `implements`/`tracks`) — assertable without network.
- [ ] Add a mocked/recorded GitHub fixture to cover valid/closed/not-found classification and the non-zero exit on 404.

## Done

- [x] Verifies `implements`/`tracks` references via `github::verify_spec_issues`, tallying valid/closed/not-found/error counts.
- [x] Repo resolution via `github::resolve_repo` with a clear error + exit 1 when unresolvable.
- [x] Text/Table/Csv, Json, and Markdown/Github output formats.
- [x] `--create` runs validation and opens drift issues for specs with errors.
- [x] Non-zero exit when any reference is not found or errored.

## Gaps

- No integration or inline unit tests target `src/commands/issues.rs`. The command depends on the live GitHub API, so end-to-end testing needs recorded fixtures or a mock.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
