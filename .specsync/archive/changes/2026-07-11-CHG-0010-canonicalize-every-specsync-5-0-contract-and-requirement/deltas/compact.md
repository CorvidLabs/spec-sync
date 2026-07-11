## ADDED

### REQUIREMENT REQ-compact-001

The compact module SHALL compact only excess Change Log rows while preserving recent entries, table structure, and dry-run safety.

Acceptance Criteria
- `compact_changelogs(root, specs_dir, keep, dry_run)` walks every spec found by `find_spec_files` and compacts each spec's `## Change Log` table
- The `## Change Log` section ends at the next `## ` heading or EOF; only that slice is rewritten
- The first two `|`-prefixed lines in the section (header + separator) are always preserved
- The last `keep` data rows are kept verbatim; the earlier `total - keep` rows are replaced by a single summary row
- The summary row reads `| <first_date> — <last_date> | Compacted: <N> entries |` for 2-column tables, and inserts a `—` placeholder for the middle column(s) on 3+ column tables
- Column count is detected from the first data row (`| count - 1`)
- If `total <= keep`, no rows are removed (`removed == 0`) and the spec is not written; such results are filtered out by `compact_changelogs`
- `dry_run: true` collects `CompactResult` values but never writes files
- Only results where `removed > 0` are returned from `compact_changelogs`
