## ADDED

### REQUIREMENT REQ-parser-001

The parser SHALL deterministically parse the supported frontmatter and Public API Markdown subset and SHALL identify required and stub sections.

Acceptance Criteria
- `parse_frontmatter` returns None if the file has no `---` delimited frontmatter block
- Handles scalar fields (module, version, status) and list fields (files, db_tables, depends_on)
- Empty list syntax `[]` is handled correctly (produces empty vec, not a vec containing "[]")
- `get_spec_symbols` extracts first backtick-quoted word from each row in `### Exported ...` subsections
- Only extracts from allowlisted subsection names (Exported Functions, Exported Types, etc.)
- Symbols are deduplicated while preserving order
- `get_missing_sections` uses case-sensitive regex matching for `## SectionName`
- Unrecognized YAML keys are silently skipped (no errors)
- Zero external YAML parsing dependencies — custom line-by-line parser

## MODIFIED

### SPEC SECTION Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `ParsedSpec` | Parsed spec file containing `frontmatter: Frontmatter` and `body: String` |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parse_frontmatter` | `content: &str` | `Option<ParsedSpec>` | Parse YAML frontmatter delimited by `---` from a spec file |
| `get_spec_symbols` | `body: &str` | `Vec<String>` | Extract backtick-quoted symbol names from the `## Public API` section tables |
| `get_missing_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Check which required `##` sections are missing from the spec body |
| `is_export_header` | `header: &str` | `bool` | Returns true if a `###` header denotes an exported-symbols subsection (e.g. `### Exported Functions`) |
| `section_has_content` | `body: &str, section: &str` | `bool` | Returns true if the `## Section` block contains non-whitespace content beyond the header line |
| `find_stub_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Returns required section names whose `## Section` blocks are present but contain no substantive content |
| `find_section_offset` | `body: &str, section: &str` | `Option<usize>` | Returns byte offset of the `## Section` heading line, using anchored regex with trailing-whitespace tolerance |
| `body_has_section` | `body: &str, section: &str` | `bool` | Returns true if the spec body contains an exact `## Section` heading (delegates to `find_section_offset`) |
| `get_near_miss_sections` | `body: &str, required_sections: &[String]` | `Vec<(String, String)>` | For each missing required section, returns `(canonical_name, found_heading)` pairs where a `## Heading` exists within Levenshtein distance ≤ 2 — used to detect typos and suggest `--fix` |
| `get_all_api_table_symbols` | `body: &str` | `Vec<String>` | Extract the first backtick-quoted symbol from every table row in `## Public API`, including informational subsections that `get_spec_symbols` skips — used by `check --fix` to avoid appending duplicate rows |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| types | `Frontmatter` struct |
| regex | `Regex`, `LazyLock` for compiled patterns |

**Consumed By**

| Module | What is used |
|--------|-------------|
| validator | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections`, `get_near_miss_sections` |
| scoring | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| commands/check | `get_near_miss_sections` (via `fix_near_miss_required_headers`) |
| mcp | `parse_frontmatter` (for listing specs) |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
