## ADDED

### REQUIREMENT REQ-importer-001

The importer SHALL normalize supported external content into safe local spec drafts while sanitizing paths, secrets, markup, and oversized input.

Acceptance Criteria
- GitHub Issues importer works with `gh` CLI and falls back to `GITHUB_TOKEN` REST API
- Jira importer supports both Atlassian Cloud (basic auth) and Server/DC (bearer token)
- Confluence importer strips HTML and extracts plain text requirements
- All imported specs have valid frontmatter and all required sections
- Requirements are automatically extracted from checkboxes and "Acceptance Criteria" / "Requirements" / "Definition of Done" sections
- Module names are properly slugified from titles (lowercased, non-alphanumerics collapsed to single `-`)
- `render_spec` emits `implements: [n]` when an issue number is present and `implements: []` otherwise (Jira/Confluence)
- Auth tokens (`JIRA_TOKEN`, `CONFLUENCE_TOKEN`, `GITHUB_TOKEN`) are redacted from REST error messages via `redact_secret` before being surfaced
