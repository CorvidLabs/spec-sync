---
spec: importer.spec.md
---

## Key Decisions

- Reuses `github::fetch_issue_details()` for explicit-token typed REST, operation bounds, strict
  payload parsing, and repository-aware 404 classification; GitHub imports never launch `gh`.
- Uses simple regex-free HTML stripping for Confluence — no external HTML parser dependency
- Base64 encoding is hand-rolled to avoid adding a dependency (only used for Jira/Confluence basic auth)
- Requirements extraction is heuristic-based: looks for checkboxes, "Acceptance Criteria"/"Requirements", and "Definition of Done" sections
- Generated specs always start as `draft` status — user fills in details after import
- `redact_secret` strips any verbatim auth token from REST error strings before surfacing them (added 4.3.5), mirroring the GitHub module's `redact_token` and the AI provider client's sanitization

## Files to Read First

- `src/importer.rs` — all importer logic, parsers, and tests
- `src/commands/import.rs` — wires the `specsync import` subcommand to the importer functions (dispatched from `src/main.rs`)

## Current Status

All three importers are implemented. The GitHub public entry delegates through an injected
crate-private seam to the shared typed provider, with success conversion and provider-failure
non-production covered in unit tests; single and batch CLI token failures are covered end-to-end
and assert that no spec output is created. GitHub issue titles that normalize to an empty safe
module name are rejected before an imported item or output path is created. Live REST success
remains an integration-only gate.

## Notes

- Jira Cloud uses email:token basic auth; Jira Server/DC uses bearer token
- Confluence storage format is HTML-like, not markdown
- Repository selection remains command-layer policy; the importer accepts a validated repository identifier and performs no Git or provider-subprocess discovery
