---
spec: parser.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Support nested YAML in frontmatter (e.g., `roles: { agent: [...], developer: [...] }`)
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
- [x] Reject duplicate frontmatter keys with offending-line diagnostics
- [x] Parse flow-style string lists and reject malformed list shapes
- [x] Warn on non-numeric versions, scalar list fields, and colon-less garbage
- [x] Tolerate a leading UTF-8 BOM

## Gaps

- YAML parsing only handles the subset used in specs — nested objects, anchors/aliases, and flow mappings are unsupported
- Full YAML typing remains out of scope; the supported flat subset has explicit shape diagnostics

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
