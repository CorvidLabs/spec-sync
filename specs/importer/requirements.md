---
spec: importer.spec.md
---

## User Stories

- As a team adopting spec-sync, I want to import existing Jira issues as spec files so that I don't have to rewrite everything manually
- As a developer, I want to import a GitHub issue into a spec so that the spec is automatically linked via `implements`
- As a team lead, I want to import Confluence pages as specs so that existing documentation is migrated into the spec system

## Acceptance Criteria

- GitHub Issues importer requires `GITHUB_TOKEN`, uses typed bounded in-process REST, revalidates
  repository access after ambiguous 404, and never launches `gh issue view`.
- Jira importer supports both Atlassian Cloud (basic auth) and Server/DC (bearer token)
- Confluence importer strips HTML and extracts plain text requirements
- All imported specs have valid frontmatter and all required sections
- Requirements are automatically extracted from checkboxes and "Acceptance Criteria" / "Requirements" / "Definition of Done" sections
- Module names are properly slugified from titles (lowercased, non-alphanumerics collapsed to single `-`)
- `render_spec` emits `implements: [n]` when an issue number is present and `implements: []` otherwise (Jira/Confluence)
- Auth tokens (`JIRA_TOKEN`, `CONFLUENCE_TOKEN`, `GITHUB_TOKEN`) are redacted from REST error messages via `redact_secret` before being surfaced

## Constraints

- No new external dependencies (uses existing `ureq` and `serde_json`)
- HTTP timeouts: 10s for GitHub, 15s for Jira/Confluence

## Out of Scope

- Batch importing multiple issues at once (future enhancement)
- Two-way sync (spec changes pushed back to Jira/Confluence)
- OAuth flows for authentication (uses explicit tokens)

### REQ-importer-001

The importer SHALL normalize supported external content into safe local spec drafts while sanitizing paths, secrets, markup, and oversized input.

Acceptance Criteria
- GitHub Issues importer requires `GITHUB_TOKEN`, uses typed bounded in-process REST, revalidates
  repository access after ambiguous 404, and never launches `gh issue view`.
- Jira importer supports both Atlassian Cloud (basic auth) and Server/DC (bearer token)
- Confluence importer strips HTML and extracts plain text requirements
- All imported specs have valid frontmatter and all required sections
- Requirements are automatically extracted from checkboxes and "Acceptance Criteria" / "Requirements" / "Definition of Done" sections
- Module names are properly slugified from titles (lowercased, non-alphanumerics collapsed to single `-`)
- `render_spec` emits `implements: [n]` when an issue number is present and `implements: []` otherwise (Jira/Confluence)
- Auth tokens (`JIRA_TOKEN`, `CONFLUENCE_TOKEN`, `GITHUB_TOKEN`) are redacted from REST error messages via `redact_secret` before being surfaced

