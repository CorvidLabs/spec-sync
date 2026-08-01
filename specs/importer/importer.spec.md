---
module: importer
version: 8
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

Generates spec files from external project management systems. Supports importing from GitHub Issues, Jira issues/epics, and Confluence pages, converting them into spec-format markdown with frontmatter, requirements or guided requirement prompts, starter sections, and traceability links.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `import_github_issue` | `repo: &str, number: u64` | `Result<ImportedItem, String>` | Fetch a GitHub issue through typed in-process REST with explicit `GITHUB_TOKEN` and convert it to `ImportedItem` |
| `import_jira_issue` | `issue_key: &str` | `Result<ImportedItem, String>` | Fetch a Jira issue via REST API v3 and convert to `ImportedItem` |
| `import_confluence_page` | `page_id: &str` | `Result<ImportedItem, String>` | Fetch a Confluence page via REST API and convert to `ImportedItem` |
| `render_spec` | `item: &ImportedItem` | `String` | Render an `ImportedItem` into a complete spec markdown string |
| `extract_requirements_pub` | `body: &str` | `Vec<String>` | Extract requirement-like bullets (checkboxes, numbered lists, acceptance criteria sections) from text |
| `extract_requirements` | `body: &str` | `Vec<String>` | Internal extractor used by `extract_requirements_pub` |
| `slugify` | `title: &str` | `String` | Convert a title into a valid module name (lowercase, hyphen-separated) |

### Exported Structs

| Type | Description |
|------|-------------|
| `ImportedItem` | Intermediate representation: `module_name`, `purpose`, `requirements`, `labels`, `source_url`, `issue_number`, `source_type` |
| `ImportSource` | Enum: `GitHub`, `Jira`, `Confluence` |

## Invariants

1. `import_github_issue` delegates to `github::fetch_issue_details`.
2. Missing tokens, malformed payloads, transport failures, timeouts, and inaccessible repositories
   are errors rather than partial imported items.
3. The issue identity, title, body, labels, state, and URL are parsed through the shared typed GitHub
   response contract.
4. `gh` remains outside the importer read path.
5. Every provider-derived module name is portable after safe-name normalization, including
   reserved-device and generated-filename byte limits.

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
| GitHub: `GITHUB_TOKEN` missing | `import_github_issue` returns an actionable `Err` without consulting authenticated `gh` state |
| Issue/page not found (404) | Each importer returns `Err("{type} not found")` |
| Network timeout | Returns `Err` with connection details |
| Invalid issue number for GitHub | CLI rejects before calling importer |
| Any imported title produces an empty, reserved, trailing-dot/space, or overlong module candidate | The GitHub/Jira/Confluence parser returns `Err` before any item, spec path, or file is created |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| github | `fetch_issue_details` for typed, bounded, repository-aware in-process REST reads |
| (external) | `ureq` for HTTP REST API calls |
| (external) | `serde_json` for parsing JSON responses |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `cmd_import` dispatches to `import_github_issue`, `import_jira_issue`, `import_confluence_page`, then `render_spec` |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-07 | Replace imported-spec unfinished-work markers with guided requirement prompts |
| 2026-07-22 | CHG-0063: Move GitHub imports to explicit-token typed REST and remove `gh issue view` reads |
| 2026-07-22 | CHG-0063 final agent review: reject GitHub titles that cannot produce a safe non-empty module name |
| 2026-07-23 | CHG-0063 portable-import follow-up: validate GitHub, Jira, and Confluence slugs against shared reserved-name and generated-filename limits |
| 2026-04-07 | Initial implementation — GitHub, Jira, Confluence importers (#97) |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
