## MODIFIED

### REQUIREMENT REQ-parser-001

The parser SHALL deterministically parse the supported frontmatter and Public API Markdown subset and SHALL identify required and stub sections.

Acceptance Criteria
- `parse_frontmatter` returns None if the file has no `---` delimited frontmatter block
- Handles scalar fields (module, version, status) and list fields (files, db_tables, depends_on)
- Empty list syntax `[]` is handled correctly (produces empty vec, not a vec containing "[]")
- `get_spec_symbols` extracts the complete first nonempty backtick-delimited symbol from each recognized table row in `### Exported ...` subsections
- Punctuation emitted by language extractors is preserved without a second parser-specific character allowlist
- Empty or malformed backtick cells do not produce symbols
- Only extracts from allowlisted subsection names (Exported Functions, Exported Types, etc.)
- Symbols are deduplicated while preserving order
- `get_missing_sections` uses case-sensitive regex matching for `## SectionName`
- Unrecognized YAML keys are silently skipped (no errors)
- Zero external YAML parsing dependencies — custom line-by-line parser
