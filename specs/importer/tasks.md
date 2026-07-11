---
spec: importer.spec.md
---

## Post-5.0 Roadmap

- [ ] Add Confluence fixtures with malformed HTML and links that should be stripped without losing visible text.

## Post-5.0 Test Debt

- [ ] Add fixture coverage for large imported issue bodies that approach spec generation limits.
- [ ] Add parser tests for Jira ADF documents with nested lists, code blocks, and unsupported node types.
- [ ] Add GitHub fallback tests for non-JSON `gh` failures and API rate-limit responses.

## Done

- [x] Implement GitHub Issues importer with `gh` CLI + REST API fallback
- [x] Implement Jira importer with ADF and plain text description support
- [x] Implement Confluence importer with HTML stripping
- [x] Add requirement extraction from checkboxes and criteria sections
- [x] Add `specsync import` CLI subcommand (`src/commands/import.rs`)
- [x] Redact auth tokens from REST error messages via `redact_secret` (4.3.5)
- [x] Write unit tests for all parsers and helpers

## Gaps


## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
