---
module: parser
version: 1
status: stable
files:
  - src/parser.rs
db_tables: []
tracks: [117]
depends_on:
  - specs/types/types.spec.md
---

# Parser

## Purpose

Parses spec markdown files — extracts YAML frontmatter into structured data, extracts backtick-quoted symbol names from Public API tables, and checks for required markdown sections. Uses zero-dependency YAML parsing (regex-based, no YAML library).

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `ParsedSpec` | Parsed spec file containing `frontmatter: Frontmatter` and `body: String` |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parse_frontmatter` | `content: &str` | `Option<ParsedSpec>` | Parse YAML frontmatter delimited by `---` from a spec file |
| `get_spec_symbols` | `body: &str` | `Vec<String>` | Extract backtick-quoted symbol names from the `## Public API` section tables |
| `get_missing_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Check which required `##` sections are missing from the spec body |
| `is_export_header` | `header: &str` | `bool` | Returns true if a `###` header denotes an exported-symbols subsection (e.g. `### Exported Functions`) |
| `section_has_content` | `body: &str, section: &str` | `bool` | Returns true if the `## Section` block contains non-whitespace content beyond the header line |
| `find_stub_sections` | `body: &str` | `Vec<String>` | Returns section names whose `## Section` blocks are present but contain no substantive content |
| `find_section_offset` | `body: &str, section: &str` | `Option<usize>` | Returns byte offset of the `## Section` heading line, using anchored regex with trailing-whitespace tolerance |
| `body_has_section` | `body: &str, section: &str` | `bool` | Returns true if the spec body contains an exact `## Section` heading (delegates to `find_section_offset`) |
| `get_near_miss_sections` | `body: &str, required_sections: &[String]` | `Vec<(String, String)>` | For each missing required section, returns `(canonical_name, found_heading)` pairs where a `## Heading` exists within Levenshtein distance ≤ 2 — used to detect typos and suggest `--fix` |

## Invariants

1. `parse_frontmatter` returns `None` if the content does not start with `---\n...\n---\n`
2. `get_spec_symbols` only extracts the first backtick-quoted word per table row (`` `symbol` ``)
3. `get_spec_symbols` only extracts from `### Exported ...` subsections (allowlist) and top-level tables; skips non-export subsections (e.g., `### API Endpoints`, `### Route Handlers`, `### Configuration`) and `####` method/constructor/properties sub-tables
4. Symbols are deduplicated while preserving order
5. `get_missing_sections` uses regex matching for `## SectionName` headings — case-sensitive
8. `get_near_miss_sections` only reports sections that are already in `get_missing_sections` — it does not flag sections that are present but close to another required name
6. Frontmatter parsing handles both scalar fields (module, version, status) and list fields (files, db_tables, depends_on)
7. Empty list syntax `[]` is handled correctly, producing an empty Vec

## Behavioral Examples

### Scenario: Parse valid frontmatter

- **Given** a spec file with `---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.ts\n---\n`
- **When** `parse_frontmatter(content)` is called
- **Then** returns `Some(ParsedSpec)` with module="auth", version="1", files=["src/auth.ts"]

### Scenario: No frontmatter delimiters

- **Given** a plain markdown file without `---` delimiters
- **When** `parse_frontmatter(content)` is called
- **Then** returns `None`

### Scenario: Extract symbols from Public API

- **Given** a spec body with a table row `| \`createAuth\` | config | Auth | Creates auth |`
- **When** `get_spec_symbols(body)` is called
- **Then** includes "createAuth" in the returned vector

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No frontmatter delimiters | `parse_frontmatter` returns `None` |
| Malformed YAML in frontmatter | Unknown keys silently ignored, missing fields remain as `None` |
| No `## Public API` section | `get_spec_symbols` returns empty vector |
| Empty body | `get_missing_sections` reports all required sections as missing |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `Frontmatter` struct |
| regex | `Regex`, `LazyLock` for compiled patterns |

### Consumed By

| Module | What is used |
|--------|-------------|
| validator | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections`, `get_near_miss_sections` |
| scoring | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| commands/check | `get_near_miss_sections` (via `fix_near_miss_required_headers`) |
| mcp | `parse_frontmatter` (for listing specs) |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
