---
spec: parser.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Decide whether the compatibility `parse_frontmatter` API should gain broader YAML support;
  checked issue-reference parsing already accepts and safely ignores nested extension YAML.
- [ ] Handle multi-line string values in frontmatter (e.g., `description: |`)
- [ ] Extract symbols from non-table Public API formats (e.g., bullet lists, code blocks)

## Done

- [x] Preserve complete dotted, hyphenated, operator, selector, Unicode, and space-containing symbols from the first Public API table cell
- [x] Zero-dependency YAML frontmatter parsing
- [x] Flat key-value and list field extraction
- [x] Inline array syntax (`[a, b]`) and multi-line list syntax
- [x] Backtick-quoted symbol extraction from markdown tables
- [x] Sub-table skipping (Methods, Constructor, Properties)
- [x] Required section presence checking
- [x] Symbol deduplication with order preservation
- [x] Add maintained real-YAML checked parsing for top-level issue references.
- [x] Reject duplicate/global malformed YAML and blank/null/wrong issue-reference shapes.
- [x] Accept YAML comments and valid trailing commas while ignoring nested extension and
  block-scalar issue-key lookalikes.
- [x] Accept CRLF checked frontmatter delimiters equivalently to LF.

## Gaps

- Compatibility `parse_frontmatter` still handles only the established subset and does not validate
  every metadata field type.
- Full-schema validation for non-issue metadata remains owned by downstream validators.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
