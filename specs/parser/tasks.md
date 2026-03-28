---
spec: parser.spec.md
---

## Tasks

- [ ] Support nested YAML in frontmatter (e.g., `roles: { agent: [...], developer: [...] }`)
- [ ] Handle multi-line string values in frontmatter (e.g., `description: |`)
- [ ] Extract symbols from non-table Public API formats (e.g., bullet lists, code blocks)

## Done

- [x] Zero-dependency YAML frontmatter parsing
- [x] Flat key-value and list field extraction
- [x] Inline array syntax (`[a, b]`) and multi-line list syntax
- [x] Backtick-quoted symbol extraction from markdown tables
- [x] Sub-table skipping (Methods, Constructor, Properties)
- [x] Required section presence checking
- [x] Symbol deduplication with order preservation

## Gaps

- YAML parsing only handles the subset used in specs — nested objects, anchors/aliases, and flow mappings are unsupported
- No validation of frontmatter field types (e.g., `version` as a number vs string)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
