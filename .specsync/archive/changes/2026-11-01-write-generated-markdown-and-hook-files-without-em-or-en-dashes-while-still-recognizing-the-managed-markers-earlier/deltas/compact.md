## MODIFIED

### REQUIREMENT REQ-compact-001

The compact module SHALL compact only excess Change Log rows while preserving recent entries, table structure, and dry-run safety.

Acceptance Criteria
- `compact_changelogs(root, specs_dir, keep, dry_run)` walks every spec found by `find_spec_files` and compacts each spec's `## Change Log` table
- The `## Change Log` section ends at the next `## ` heading or EOF; only that slice is rewritten
- The first two `|`-prefixed lines in the section (header + separator) are always preserved
- The last `keep` ordinary data rows are kept verbatim; the earlier `total - keep` rows are replaced by a single summary row
- The summary row reads `| <first_date> to <last_date> | Compacted: <N> entries <!-- specsync:compact:v1 --> |` for 2-column tables, and inserts a `-` placeholder for the middle column(s) on 3+ column tables; a written summary row contains no U+2013 or U+2014
- A generated summary row is recognized only when its first cell contains a non-empty `start to end` range, every interior cell is `-`, and its final cell contains a grammatically-correct fixed-width count plus the exact `<!-- specsync:compact:v1 -->` marker
- A summary row in the form written up to and including 6.1.1, whose range separator and every interior cell use U+2014, is recognized the same way; a row mixing the two placeholder forms is not a summary
- When new ordinary rows require another compaction, prior generated summary counts are accumulated and the original range start is retained, and the replacement summary is written in the current form whichever form the prior summary used
- Multiple marked summaries fail closed instead of being summed, whatever form each uses
- Column count and cells are parsed from one contiguous table without treating odd-backslash escaped pipes or code-span pipes as delimiters
- If `total <= keep`, no rows are removed (`removed == 0`) and the spec is not written; such results are filtered out by `compact_changelogs`
- `dry_run: true` collects `CompactResult` values but never writes files
- Only results where `removed > 0` are returned from `compact_changelogs`
- Re-running with no excess ordinary rows leaves the file byte-for-byte unchanged, including exact LF/CRLF terminators and a summary row in the 6.1.1 form
- `CompactResult.compacted_entries` reports ordinary entries retained and excludes the generated summary row
- Count overflow, malformed widths, and ambiguous generated state fail closed
- Preflight or staging failure writes nothing, and every incomplete apply is represented in the typed report
