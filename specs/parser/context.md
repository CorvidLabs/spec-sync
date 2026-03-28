---
spec: parser.spec.md
---

## Key Decisions

- **Zero-dependency YAML**: Frontmatter is parsed line-by-line with regex instead of using a YAML library. This handles the subset of YAML actually used in specs (flat key-value pairs and simple lists) without pulling in a full YAML parser.
- **First backtick per row**: Only the first backtick-quoted identifier in each markdown table row is extracted as a symbol. This matches the spec convention where the function/type name is always in the first column.
- **Sub-table skipping**: `### Methods`, `### Constructor`, `### Properties` headings inside the Public API section are skipped when extracting symbols to avoid double-counting members of a documented type.
- **Deduplication with order preservation**: Extracted symbols are deduplicated while maintaining their order of appearance in the spec.
- **Case-sensitive section matching**: Required section names are matched exactly (e.g., `## Public API` won't match `## public api`), enforcing consistent spec formatting.

## Files to Read First

- `src/parser.rs` — Single-file module with frontmatter parsing, symbol extraction, and section checking.

## Current Status

Fully implemented. The parser is the most heavily depended-on module after types — validator, scoring, and MCP all use it for reading specs.

## Notes

- The `parse_frontmatter()` function returns both the parsed `Frontmatter` struct and the body text (everything after the closing `---`). This avoids re-reading the file for section analysis.
- Frontmatter fields like `files`, `db_tables`, and `depends_on` support both inline array syntax (`[a, b]`) and multi-line YAML list syntax (`- a\n- b`).
