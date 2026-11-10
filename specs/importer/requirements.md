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
- Module names are slugified from titles and then validated against the shared portable component
  contract, including Windows device basenames and generated spec filename length.
- A GitHub, Jira, or Confluence title that cannot produce a portable module name fails before an
  `ImportedItem` or filesystem path is produced.
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

The `importer` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-importer-002

An `ImportedItem` SHALL carry the content preserved from a local markdown source — its `files`, `depends_on`, `db_tables` and `implements` lists, declared `version`, title, H1 lead-in and existing sections — and `render_spec` SHALL emit the real lists and reuse preserved sections instead of always writing a fixed skeleton with `files: []`. Rendered imports are always `status: draft`.

Acceptance Criteria
- `render_spec` writes the item's preserved frontmatter lists into the frontmatter verbatim, and only emits empty lists when the source supplied none (external GitHub, Jira and Confluence imports).
- When the item carries preserved sections, `render_spec` reuses the source's Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies and Change Log bodies, scaffolds only the sections the source lacks, keeps source sections outside the canonical set, keeps the H1 lead-in paragraph, and appends the import Change Log row to an existing Change Log instead of replacing it.
- Section splitting never treats a `## ` line inside a fenced code block as a heading.
- A spec rendered from a frontmatter-complete source has no frontmatter failure under `specsync check`; `check --strict` behaves exactly as it does for a `specsync new` scaffold (companion scaffold markers remain to be filled).
- Rendered imports always carry `status: draft`; a source's non-draft status is noted in the import Change Log row.
- `ImportSource` names a local directory import as itself; no directory import is reported as Confluence.

