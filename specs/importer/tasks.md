---
spec: importer.spec.md
---

## Post-5.0 Roadmap

- [ ] Add Confluence fixtures with malformed HTML and links that should be stripped without losing visible text.

## Post-5.0 Test Debt

- [ ] Add fixture coverage for large imported issue bodies that approach spec generation limits.
- [ ] Add parser tests for Jira ADF documents with nested lists, code blocks, and unsupported node types.
- [ ] Add live GitHub REST integration coverage for rate-limit responses without exposing credentials.

## Done

- [x] Implement GitHub Issues importer with explicit-token typed REST and no read subprocess
- [x] Implement Jira importer with ADF and plain text description support
- [x] Implement Confluence importer with HTML stripping
- [x] Add requirement extraction from checkboxes and criteria sections
- [x] Add `specsync import` CLI subcommand (`src/commands/import.rs`)
- [x] Redact auth tokens from REST error messages via `redact_secret` (4.3.5)
- [x] Write unit tests for all parsers and helpers
- [x] Exercise the GitHub importer entry path through its typed provider seam for success and
  failure without producing an item on provider error
- [x] Reject GitHub issue titles that cannot produce a safe non-empty module name
- [x] Reject non-portable GitHub, Jira, and Confluence slugs, including Windows reserved device
  basenames and overlong generated spec filenames, before producing an imported item

## Gaps


## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
