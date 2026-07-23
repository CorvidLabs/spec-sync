## ADDED

### REQUIREMENT REQ-parser-002

The parser SHALL provide checked frontmatter parsing that rejects ambiguous or malformed metadata
with deterministic, field-specific diagnostics while retaining a compatibility wrapper for
non-gating callers during migration.

Acceptance Criteria

- A checked API returns either a parsed spec or an ordered collection of diagnostics containing a
  diagnostic kind, message, optional field name, and source line when known.
- Missing or unterminated frontmatter delimiters, invalid YAML syntax, colon-less content,
  malformed quotes or flow collections, invalid indentation or tabs, and unsupported known-field
  shapes produce diagnostics instead of a partially populated `Frontmatter`.
- Duplicate top-level keys are rejected before any value can override an earlier value, including
  duplicate syntactically valid unknown extension fields.
- `module`, `status`, and `agent_policy` accept only scalar strings; `files`, `db_tables`,
  `depends_on`, and `lifecycle_log` accept only string sequences; `implements` and `tracks` accept
  only sequences of non-negative integer issue numbers.
- `version` accepts the supported generated numeric representation and valid non-empty textual
  version representation, but rejects booleans, maps, sequences, nulls, and non-version text.
- Block and flow sequences have identical semantics; `depends_on: [alpha, beta]` produces two
  declarations, while scalar and map forms fail.
- Unknown top-level extension fields remain allowed when syntactically valid and unique and do not
  alter known fields.
- Leading UTF-8 BOM handling, CRLF normalization, inline comments, deterministic order, and the
  existing Markdown body and Public API extraction behavior remain compatible.
- Gating consumers use the checked API and never use `Option`-based parsing to omit malformed
  specs; the compatibility wrapper delegates to the checked implementation.

## MODIFIED

### SPEC SECTION Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `ParsedSpec` | Parsed spec file containing `frontmatter: Frontmatter` and `body: String` |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parse_frontmatter_checked` | `content: &str` | `Result<ParsedSpec, Vec<FrontmatterDiagnostic>>` | Parse and type-check supported frontmatter, returning ordered field-specific diagnostics for malformed or ambiguous metadata |
| `parse_frontmatter` | `content: &str` | `Option<ParsedSpec>` | Compatibility wrapper over checked parsing for explicitly non-gating callers; returns `None` when checked parsing reports diagnostics |
| `get_spec_symbols` | `body: &str` | `Vec<String>` | Extract backtick-quoted symbol names from the `## Public API` section tables |
| `get_missing_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Check which required `##` sections are missing from the spec body |
| `is_export_header` | `header: &str` | `bool` | Returns true if a `###` header denotes an exported-symbols subsection (e.g. `### Exported Functions`) |
| `section_has_content` | `body: &str, section: &str` | `bool` | Returns true if the `## Section` block contains non-whitespace content beyond the header line |
| `find_stub_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Returns required section names whose `## Section` blocks are present but contain no substantive content |
| `find_section_offset` | `body: &str, section: &str` | `Option<usize>` | Returns byte offset of the `## Section` heading line, using anchored regex with trailing-whitespace tolerance |
| `body_has_section` | `body: &str, section: &str` | `bool` | Returns true if the spec body contains an exact `## Section` heading (delegates to `find_section_offset`) |
| `get_near_miss_sections` | `body: &str, required_sections: &[String]` | `Vec<(String, String)>` | For each missing required section, returns `(canonical_name, found_heading)` pairs where a `## Heading` exists within Levenshtein distance ≤ 2 |
| `get_all_api_table_symbols` | `body: &str` | `Vec<String>` | Extract the first backtick-quoted symbol from every table row in `## Public API`, including informational subsections skipped by `get_spec_symbols` |

### SPEC SECTION Invariants

1. `parse_frontmatter_checked` either returns a complete typed `ParsedSpec` or an ordered diagnostic
   collection; it never returns partially accepted frontmatter.
2. Duplicate known or extension keys, malformed delimiters or YAML syntax, invalid indentation or
   tabs, and invalid known-field types, shapes, or values are checked errors.
3. Known scalar, string-sequence, and issue-number-sequence fields accept only their declared
   shapes; block and flow sequences have identical semantics.
4. Syntactically valid unique unknown top-level extension fields are accepted without changing
   `Frontmatter`.
5. The parser preserves leading-BOM and CRLF compatibility, inline comments, declaration order,
   Markdown body behavior, and deterministic diagnostics.
6. `parse_frontmatter` delegates to checked parsing and is restricted to non-gating compatibility
   paths.
7. `get_spec_symbols` extracts only complete first-cell backtick symbols from recognized exported
   API tables, preserves extractor punctuation, and deduplicates in declaration order.
8. `get_missing_sections` remains case-sensitive, and near-miss detection reports only required
   sections that are actually missing.
