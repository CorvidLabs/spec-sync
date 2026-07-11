---
spec: parser.spec.md
---

## User Stories

- As the validator module, I need parsed frontmatter from spec files so that I can check module metadata (name, version, status, files, dependencies)
- As the scoring module, I need extracted symbols from Public API tables so that I can measure API documentation coverage
- As the validator module, I need to check which required sections are present in a spec so that I can report missing sections as errors
- As a developer writing specs, I want lenient YAML parsing so that minor formatting issues don't cause hard failures

## Acceptance Criteria

- `parse_frontmatter` returns None if the file has no `---` delimited frontmatter block
- Handles scalar fields (module, version, status) and list fields (files, db_tables, depends_on)
- Empty list syntax `[]` is handled correctly (produces empty vec, not a vec containing "[]")
- `get_spec_symbols` extracts first backtick-quoted word from each row in `### Exported ...` subsections
- Only extracts from allowlisted subsection names (Exported Functions, Exported Types, etc.)
- Symbols are deduplicated while preserving order
- `get_missing_sections` uses case-sensitive regex matching for `## SectionName`
- Unrecognized YAML keys are silently skipped (no errors)
- Zero external YAML parsing dependencies — custom line-by-line parser

## Constraints

- Parser must be deterministic — same input always produces same output
- Must not allocate excessively for large spec files
- Frontmatter parsing must handle edge cases (missing colons, extra whitespace, inline comments)

## Out of Scope

- Full YAML spec compliance (only the subset used in spec frontmatter)
- Parsing non-spec markdown files
- Modifying or writing spec files (parser is read-only)
- Validating frontmatter values (that's the validator's job)

### REQ-parser-001

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

