---
spec: importer.spec.md
---

## Done

- [x] Implement GitHub Issues importer with `gh` CLI + REST API fallback
- [x] Implement Jira importer with ADF and plain text description support
- [x] Implement Confluence importer with HTML stripping
- [x] Add requirement extraction from checkboxes and criteria sections
- [x] Add `specsync import` CLI subcommand (`src/commands/import.rs`)
- [x] Redact auth tokens from REST error messages via `redact_secret` (4.3.5)
- [x] Write unit tests for all parsers and helpers

## Gaps

- [ ] Add fixture coverage for large imported issue bodies that approach spec generation limits.
- [ ] Add parser tests for Jira ADF documents with nested lists, code blocks, and unsupported node types.
- [ ] Add Confluence fixtures with malformed HTML and links that should be stripped without losing visible text.
- [ ] Add GitHub fallback tests for non-JSON `gh` failures and API rate-limit responses.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
