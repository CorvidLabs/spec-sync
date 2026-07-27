## MODIFIED

### REQUIREMENT REQ-compact-001

The compact module SHALL compact only excess Change Log rows while preserving recent entries, table structure, and dry-run safety.

Acceptance Criteria
- `compact_changelogs(root, specs_dir, keep, dry_run)` walks every spec found by `find_spec_files` and compacts each spec's `## Change Log` table
- The `## Change Log` section ends at the next `## ` heading or EOF; only that slice is rewritten
- The first two `|`-prefixed lines in the section (header + separator) are always preserved
- The last `keep` ordinary data rows are kept verbatim; the earlier `total - keep` rows are replaced by a single summary row
- The summary row reads `| <first_date> — <last_date> | Compacted: <N> entries |` for 2-column tables, and inserts a `—` placeholder for the middle column(s) on 3+ column tables
- A generated summary row is recognized only when its first cell contains a non-empty `start — end` range, every interior cell is `—`, and its final cell contains a grammatically-correct fixed-width count plus the exact `<!-- specsync:compact:v1 -->` marker
- When new ordinary rows require another compaction, prior generated summary counts are accumulated and the original range start is retained
- Multiple marked summaries fail closed instead of being summed
- Column count and cells are parsed from one contiguous table without treating odd-backslash escaped pipes or code-span pipes as delimiters
- If `total <= keep`, no rows are removed (`removed == 0`) and the spec is not written; such results are filtered out by `compact_changelogs`
- `dry_run: true` collects `CompactResult` values but never writes files
- Only results where `removed > 0` are returned from `compact_changelogs`
- Re-running with no excess ordinary rows leaves the file byte-for-byte unchanged, including exact LF/CRLF terminators
- `CompactResult.compacted_entries` reports ordinary entries retained and excludes the generated summary row
- Count overflow, malformed widths, and ambiguous generated state fail closed
- Preflight or staging failure writes nothing, and every incomplete apply is represented in the typed report

### SPEC SECTION Invariants

1. Only an exact `## Change Log` H2 outside fenced and indented code is processed
2. Only the first contiguous, width-valid table outside fenced and indented code in the section is compacted; later tables are preserved
3. The last `keep` data rows are preserved; earlier rows are summarized
4. Summary row contains the date range of compacted entries and their count
5. If a changelog has fewer than `keep + 1` entries, no compaction occurs
6. `dry_run: true` returns results without modifying files
7. Handles both 2-column and 3+ column tables with appropriate summary format
8. Re-running compaction with the same `keep` value is byte-for-byte idempotent.
9. Escaped table pipes, code-span pipes, every original LF/CRLF line terminator, and the original final-newline state are preserved
10. Only rows carrying the exact `<!-- specsync:compact:v1 -->` provenance marker are folded as prior summaries
11. Multiple marked summaries, malformed table widths, and fixed-width count overflow fail closed
12. Apply mode preflights every replacement and stages same-directory temporary files before publication
13. Staging failures retain every planned result/count with zero writes; late publish failures retain all results and report exact partial progress
14. Indented pipe code terminates table data, indented separators fail before writes, and a generated summary is valid only as the first data row
