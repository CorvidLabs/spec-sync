---
spec: parser.spec.md
---

## Key Decisions

- **Zero-dependency YAML**: Frontmatter is parsed line-by-line with regex instead of using a YAML library. This handles the subset of YAML actually used in specs (flat key-value pairs and simple lists) without pulling in a full YAML parser.
- **First code span in the first cell**: Only one complete nonempty backtick-delimited symbol occupying the first Markdown table cell is extracted. The parser does not maintain a character allowlist because extractors emit dotted YAML paths, selectors, operators, apostrophes, Unicode, and quoted names with spaces. Empty, unterminated, later-column, and prose spans remain excluded.
- **Sub-table skipping**: `####` headings containing `Methods`, `Constructor`, or `Properties` inside the Public API section are skipped when extracting symbols to avoid double-counting members of a documented type. In addition, `###` subsections that are not export headers (e.g. `### API Endpoints`, `### Route Handlers`, `### Configuration`) are skipped via an `is_export_header` allowlist.
- **Deduplication with order preservation**: Extracted symbols are deduplicated while maintaining their order of appearance in the spec.
- **Case-sensitive section matching**: Required section names are matched exactly (e.g., `## Public API` won't match `## public api`), enforcing consistent spec formatting.
- **Checked subset parsing**: The zero-dependency parser still accepts only the documented flat subset, but `ParsedSpec.errors`/`warnings` preserve duplicate keys, invalid list shapes, suspicious versions, and malformed lines instead of silently dropping them.

## Files to Read First

- `src/parser.rs` — Single-file module with frontmatter parsing, symbol extraction, and section checking.

## Current Status

Fully implemented. The parser is the most heavily depended-on module after types — validator, scoring, and MCP all use it for reading specs. Public API symbol parsing preserves exact extractor spelling across all supported languages.

## Notes

- The `parse_frontmatter()` function returns both the parsed `Frontmatter` struct and the body text (everything after the closing `---`). This avoids re-reading the file for section analysis.
- Frontmatter fields like `files`, `db_tables`, and `depends_on` support both inline array syntax (`[a, b]`) and multi-line YAML list syntax (`- a\n- b`).
- Duplicate keys remain diagnosable even though parsing continues to accumulate other findings; validator failure is therefore independent of which duplicate value appeared last.
