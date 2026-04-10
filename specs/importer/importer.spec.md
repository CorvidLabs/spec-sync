---
module: importer
version: 1
status: active
files:
  - src/importer.rs
db_tables: []
implements: [97]
depends_on:
  - specs/github/github.spec.md
---

# Importer

## Purpose

Generates spec files from external project management systems. Supports importing from GitHub Issues, Jira issues/epics, and Confluence pages, converting them into spec-format markdown with frontmatter, requirements, and traceability links.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `import_github_issue` | `repo: &str, number: u64` | `Result<ImportedItem, String>` | Fetch a GitHub issue and convert to `ImportedItem`; tries `gh` CLI, falls back to REST API |
| `import_jira_issue` | `issue_key: &str` | `Result<ImportedItem, String>` | Fetch a Jira issue via REST API v3 and convert to `ImportedItem` |
| `import_confluence_page` | `page_id: &str` | `Result<ImportedItem, String>` | Fetch a Confluence page via REST API and convert to `ImportedItem` |
| `render_spec` | `item: &ImportedItem` | `String` | Render an `ImportedItem` into a complete spec markdown string |
| `extract_requirements_pub` | `body: &str` | `Vec<String>` | Extract requirement-like bullets (checkboxes, numbered lists, acceptance criteria sections) from text |
| `slugify` | `title: &str` | `String` | Convert a title into a valid module name (lowercase, hyphen-separated) |

### Exported Structs

| Type | Description |
|------|-------------|
| `ImportedItem` | Intermediate representation: `module_name`, `purpose`, `requirements`, `labels`, `source_url`, `issue_number`, `source_type` |
| `ImportSource` | Enum: `GitHub`, `Jira`, `Confluence` |

## Invariants

1. `import_github_issue` follows the same auth strategy as `github::fetch_issue` — `gh` CLI first, REST API fallback
2. Jira importer handles both ADF (Atlassian Document Format) and plain text descriptions
3. Confluence importer strips HTML tags to extract plain text from storage format
4. `slugify` always produces a valid, non-empty module name from non-empty input (lowercase, no special chars)
5. `render_spec` always produces valid spec frontmatter with all required fields
6. Requirements are extracted from markdown checkboxes, "Acceptance Criteria" sections, and "Definition of Done" sections
7. Jira auth supports both Cloud (email:token basic auth) and Server/DC (bearer token)
8. Confluence auth supports both Cloud (email:token basic auth) and Server/DC (bearer token)
9. HTTP timeouts are 10s for GitHub, 15s for Jira and Confluence
10. Generated specs always have `status: draft` and `version: 1`

## Behavioral Examples

### Scenario: Import GitHub issue with acceptance criteria

- **Given** GitHub issue #42 titled "Add user auth" with body containing checkboxes
- **When** `import_github_issue("org/repo", 42)` is called
- **Then** returns `ImportedItem` with `module_name: "add-user-auth"`, `issue_number: Some(42)`, and extracted requirements from checkboxes

### Scenario: Import Jira issue with ADF description

- **Given** Jira issue `PROJ-123` with ADF-format description containing acceptance criteria
- **When** `import_jira_issue("PROJ-123")` is called
- **Then** extracts text from ADF content tree and parses requirements

### Scenario: Import Confluence page

- **Given** Confluence page ID `98765` with HTML storage body
- **When** `import_confluence_page("98765")` is called
- **Then** strips HTML, extracts purpose from first line, and parses requirements

### Scenario: Render spec with issue number

- **Given** an `ImportedItem` with `issue_number: Some(42)`
- **When** `render_spec(&item)` is called
- **Then** generated frontmatter contains `implements: [42]`

### Scenario: Render spec without issue number

- **Given** an `ImportedItem` with `issue_number: None` (Jira/Confluence)
- **When** `render_spec(&item)` is called
- **Then** generated frontmatter contains `implements: []`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `JIRA_URL` not set | `import_jira_issue` returns `Err("JIRA_URL environment variable not set")` |
| `JIRA_TOKEN` not set | `import_jira_issue` returns `Err("JIRA_TOKEN environment variable not set")` |
| `CONFLUENCE_URL` not set | `import_confluence_page` returns `Err("CONFLUENCE_URL environment variable not set")` |
| `CONFLUENCE_TOKEN` not set | `import_confluence_page` returns `Err("CONFLUENCE_TOKEN environment variable not set")` |
| GitHub: neither `gh` nor `GITHUB_TOKEN` | `import_github_issue` returns `Err` |
| Issue/page not found (404) | Each importer returns `Err("{type} not found")` |
| Network timeout | Returns `Err` with connection details |
| Invalid issue number for GitHub | CLI rejects before calling importer |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| github | `gh_is_available` for auth detection |
| (external) | `ureq` for HTTP REST API calls |
| (external) | `serde_json` for parsing JSON responses |
| (external) | `gh` CLI for authenticated GitHub operations |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `cmd_import` dispatches to `import_github_issue`, `import_jira_issue`, `import_confluence_page`, then `render_spec` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-07 | Initial implementation — GitHub, Jira, Confluence importers (#97) |
