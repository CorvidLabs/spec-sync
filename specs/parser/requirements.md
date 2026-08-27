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

The `parser` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-parser-002

Frontmatter parsing SHALL resolve a single- or double-quoted block list item or scalar to
the text inside the quotes, for every field.

Acceptance Criteria
- Quoted entries in `files:`, `depends_on:`, and `db_tables:` resolve to the path inside the quotes, for single and double quotes, mixed with unquoted entries in the same list.
- Quoted scalars such as `module:` and `status:` resolve to the text inside the quotes.
- A comment following the closing quote is discarded; a `#` inside the quotes is retained as content.
- An opening quote with no matching close is a frontmatter error naming the offending value, and the value is not retained as a literal.
- Flow-style lists continue to unquote their own items.

### REQ-parser-003

Every frontmatter reader in this module SHALL recognize a delimiter LINE by one rule — exactly
three dashes followed by nothing but whitespace — applied to BOTH ends of the block, in either
line encoding, with the two ends free to disagree with each other.

Acceptance Criteria
- A delimiter carrying trailing spaces or tabs opens and closes frontmatter in `strip_frontmatter`,
  `parse_frontmatter`, and `parse_checked_issue_references` alike, so a document with a padded
  OPENER has its YAML removed from the body rather than counted as prose, and a document with a
  padded CLOSER keeps the body prose above the first horizontal rule below it.
- A padded closer never lets frontmatter run into the body: `parse_frontmatter` emits no
  "Ignoring malformed frontmatter line" warning for body prose, and `parse_checked_issue_references`
  reads the references that are there instead of reporting the YAML invalid.
- `parse_checked_issue_references` reads a document whose opening and closing delimiters carry
  different line endings, which the hand-rolled pair of prefix/split chains it replaces could not.
- A line that is not exactly three dashes is not a delimiter in any reader: `----`, `--- x`,
  `---change: x`, and an indented `  ---` leave `strip_frontmatter` returning the document whole,
  `parse_frontmatter` returning `None`, and `parse_checked_issue_references` returning its stable
  content-free error. Loosening this would cut the body of any document that opens with a Markdown
  thematic break at its next rule.
- The three readers return the same verdict for every delimiter shape, asserted as a matrix, so the
  rule cannot be loosened in one reader and not the others.
- `parse_checked_issue_references` keeps the verdicts it had for an empty frontmatter block and for
  a block that is a single blank line.
- `parse_frontmatter` returns an LF-only body when the frontmatter is LF and only the body is CRLF.

