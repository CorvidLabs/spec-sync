---
spec: cmd_issues.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [x] Add command-level coverage for the no-references path with and without configured `github.repo`.
- [ ] Add a mocked/recorded GitHub fixture to cover valid/closed/not-found classification and the non-zero exit on 404.

## Done

- [x] Verifies all `implements`/`tracks` references through one bounded globally deduplicated batch,
  tallying valid/closed/not-found/error counts per spec.
- [x] Repo resolution via `github::resolve_repo` with a clear error + exit 1 when unresolvable.
- [x] Text/Table/Csv, Json, and Markdown/Github output formats.
- [x] `--create` runs validation and opens drift issues for specs with errors.
- [x] Non-zero exit when any reference is not found or errored.
- [x] Gather references before repository/provider resolution and skip GitHub entirely when empty.
- [x] Add a command-level missing-token regression with per-spec JSON error attribution.

## Gaps

- Network-free command fixtures cover the no-reference path; end-to-end provider classification
  still needs recorded fixtures or a mock process boundary.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
