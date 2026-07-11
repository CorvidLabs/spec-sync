---
spec: compact.spec.md
---

## User Stories

- As a maintainer of long-lived specs, I want old changelog rows collapsed into one summary line so that the `## Change Log` table stays readable
- As a CI operator, I want a `--dry-run` mode so that I can preview which specs would change before writing anything
- As a developer, I want compaction to keep the most recent N entries verbatim so that recent history is never lost

## Acceptance Criteria

- `compact_changelogs(root, specs_dir, keep, dry_run)` walks every spec found by `find_spec_files` and compacts each spec's `## Change Log` table
- The `## Change Log` section ends at the next `## ` heading or EOF; only that slice is rewritten
- The first two `|`-prefixed lines in the section (header + separator) are always preserved
- The last `keep` data rows are kept verbatim; the earlier `total - keep` rows are replaced by a single summary row
- The summary row reads `| <first_date> — <last_date> | Compacted: <N> entries |` for 2-column tables, and inserts a `—` placeholder for the middle column(s) on 3+ column tables
- Column count is detected from the first data row (`| count - 1`)
- If `total <= keep`, no rows are removed (`removed == 0`) and the spec is not written; such results are filtered out by `compact_changelogs`
- `dry_run: true` collects `CompactResult` values but never writes files
- Only results where `removed > 0` are returned from `compact_changelogs`

## Constraints

- Operates on the raw markdown text via string slicing; no markdown AST
- A spec is only touched if it contains a `## Change Log` marker; otherwise silently skipped
- Date range is taken from the first cell of the first/last removed rows (`extract_first_cell`)
- Write failures print a bold-red `error:` line to stderr and continue with the next spec

## Out of Scope

- Compacting any section other than `## Change Log`
- Parsing or validating individual changelog dates
- Merging or de-duplicating changelog content beyond counting removed rows
- Restoring previously compacted entries
