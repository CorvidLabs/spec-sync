---
spec: parser.spec.md
---

## User Stories

- As the validator module, I need parsed frontmatter from spec files so that I can check module metadata (name, version, status, files, dependencies)
- As the scoring module, I need extracted symbols from Public API tables so that I can measure API documentation coverage
- As the validator module, I need to check which required sections are present in a spec so that I can report missing sections as errors
- As a compatibility caller, I want the established supported-subset parser to remain available for
  ordinary spec metadata.
- As an issue-verification caller, I want complete real-YAML validation so malformed or ambiguous
  frontmatter cannot silently become an empty reference set.

## Acceptance Criteria

- `parse_frontmatter` returns None if the file has no `---` delimited frontmatter block
- Handles scalar fields (module, version, status) and list fields (files, db_tables, depends_on)
- Empty list syntax `[]` is handled correctly (produces empty vec, not a vec containing "[]")
- `get_spec_symbols` extracts first backtick-quoted word from each row in `### Exported ...` subsections
- Only extracts from allowlisted subsection names (Exported Functions, Exported Types, etc.)
- Symbols are deduplicated while preserving order
- `get_missing_sections` uses case-sensitive regex matching for `## SectionName`
- The compatibility `parse_frontmatter` path silently skips unrecognized keys within its documented
  supported subset.
- `parse_checked_issue_references` uses maintained `serde-saphyr` real-YAML parsing with
  duplicate-key rejection.
- Checked issue parsing accepts LF and CRLF frontmatter delimiters equivalently.
- Checked issue parsing accepts YAML comments and valid trailing commas.
- Checked issue parsing rejects malformed YAML anywhere in frontmatter and duplicate keys anywhere
  in the YAML mapping tree.
- Top-level `implements` and `tracks` must be sequences of positive unsigned issue numbers; blank,
  null, scalar, mapping, mixed, zero, negative, and overflowing values are rejected.
- Nested extension mappings/sequences and block-scalar text containing issue-like keys are ignored
  as issue references.

## Constraints

- Parser must be deterministic — same input always produces same output
- Must not allocate excessively for large spec files
- Compatibility frontmatter parsing must retain its documented whitespace/comment behavior.
- Checked issue-reference errors must be stable and content-free.

## Out of Scope

- Expanding compatibility `parse_frontmatter` to full YAML semantics
- Parsing non-spec markdown files
- Modifying or writing spec files (parser is read-only)
- Validating frontmatter values (that's the validator's job)

### REQ-parser-001

The `parser` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change, including error reporting and coverage/enforcement edges that those fixes address.

Acceptance Criteria
- Related `cargo test` coverage for `parser` remains green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

