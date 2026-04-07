---
spec: importer.spec.md
---

## User Stories

- As a team adopting spec-sync, I want to import existing Jira issues as spec files so that I don't have to rewrite everything manually
- As a developer, I want to import a GitHub issue into a spec so that the spec is automatically linked via `implements`
- As a team lead, I want to import Confluence pages as specs so that existing documentation is migrated into the spec system

## Acceptance Criteria

- GitHub Issues importer works with `gh` CLI and falls back to `GITHUB_TOKEN` REST API
- Jira importer supports both Atlassian Cloud (basic auth) and Server/DC (bearer token)
- Confluence importer strips HTML and extracts plain text requirements
- All imported specs have valid frontmatter and all required sections
- Requirements are automatically extracted from checkboxes and acceptance criteria sections
- Module names are properly slugified from titles

## Constraints

- No new external dependencies (uses existing `ureq` and `serde_json`)
- HTTP timeouts: 10s for GitHub, 15s for Jira/Confluence

## Out of Scope

- Batch importing multiple issues at once (future enhancement)
- Two-way sync (spec changes pushed back to Jira/Confluence)
- OAuth flows for authentication (uses tokens/CLI)
