---
spec: parser.spec.md
---

## Key Decisions

- **Two explicit frontmatter contracts**: `parse_frontmatter` remains the line-oriented
  compatibility parser for the established flat metadata subset. Security-sensitive issue
  inspection uses `parse_checked_issue_references`, a maintained `serde-saphyr` real-YAML path;
  callers must not infer checked issue truth from the lenient compatibility result.
- **Fail-closed issue fields**: checked parsing rejects duplicate keys and malformed YAML anywhere,
  and requires top-level `implements`/`tracks` to be positive unsigned-number sequences. Comments
  and valid trailing commas remain valid YAML. Nested extension data and block scalars are parsed
  but cannot masquerade as authoritative issue-reference fields. Frontmatter delimiter extraction
  accepts LF and CRLF checkouts equivalently before real-YAML parsing.
- **First code span in the first cell**: Only one complete nonempty backtick-delimited symbol occupying the first Markdown table cell is extracted. The parser does not maintain a character allowlist because extractors emit dotted YAML paths, selectors, operators, apostrophes, Unicode, and quoted names with spaces. Empty, unterminated, later-column, and prose spans remain excluded.
- **Sub-table skipping**: `####` headings containing `Methods`, `Constructor`, or `Properties` inside the Public API section are skipped when extracting symbols to avoid double-counting members of a documented type. In addition, `###` subsections that are not export headers (e.g. `### API Endpoints`, `### Route Handlers`, `### Configuration`) are skipped via an `is_export_header` allowlist.
- **Deduplication with order preservation**: Extracted symbols are deduplicated while maintaining their order of appearance in the spec.
- **Case-sensitive section matching**: Required section names are matched exactly (e.g., `## Public API` won't match `## public api`), enforcing consistent spec formatting.

## Files to Read First

- `src/parser.rs` — Single-file module with frontmatter parsing, symbol extraction, and section checking.

## Current Status

CHG-0063 implementation is present. The parser remains heavily depended on by validator, scoring,
CLI issue verification, and MCP. Public API symbol parsing preserves exact extractor spelling,
while CLI/MCP issue verification share the strict checked real-YAML boundary. Fresh CHG definition
reapproval and final independent/repository gates remain pending.

## Notes

- The `parse_frontmatter()` function returns both the parsed `Frontmatter` struct and the body text (everything after the closing `---`). This avoids re-reading the file for section analysis.
- Frontmatter fields like `files`, `db_tables`, and `depends_on` support both inline array syntax (`[a, b]`) and multi-line YAML list syntax (`- a\n- b`).
- `parse_checked_issue_references()` returns `(implements, tracks)` only after the entire
  frontmatter document passes real-YAML and duplicate-key validation; surfaced errors are stable
  and content-free.
