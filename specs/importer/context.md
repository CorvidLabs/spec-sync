---
spec: importer.spec.md
---

## Key Decisions

- Reuses existing `github::gh_is_available()` for auth detection rather than duplicating
- Uses simple regex-free HTML stripping for Confluence — no external HTML parser dependency
- Base64 encoding is hand-rolled to avoid adding a dependency (only used for Jira/Confluence basic auth)
- Requirements extraction is heuristic-based: looks for checkboxes, "Acceptance Criteria", and "Definition of Done" sections
- Generated specs always start as `draft` status — user fills in details after import

## Files to Read First

- `src/importer.rs` — all importer logic, parsers, and tests
- `src/main.rs` — `cmd_import` function wires CLI to importers

## Current Status

All three importers implemented and tested. CLI subcommand wired up.

## Notes

- Jira Cloud uses email:token basic auth; Jira Server/DC uses bearer token
- Confluence storage format is HTML-like, not markdown
- GitHub importer can auto-detect repo from git remote
