---
spec: compact.spec.md
---

## User Stories

- As a maintainer of long-lived specs, I want old changelog rows collapsed into one summary line so that the `## Change Log` table stays readable
- As a CI operator, I want a `--dry-run` mode so that I can preview which specs would change before writing anything
- As a developer, I want compaction to keep the most recent N entries verbatim so that recent history is never lost
- As a maintainer, I want repeated compaction to be byte-for-byte idempotent so generated summaries never destroy their original count or range

## Acceptance Criteria

- `compact_changelogs(root, specs_dir, keep, dry_run)` walks every spec found by `find_spec_files` and compacts each spec's `## Change Log` table
- The `## Change Log` section ends at the next `## ` heading or EOF; only that slice is rewritten
- The first two `|`-prefixed lines in the section (header + separator) are always preserved
- The last `keep` ordinary data rows are kept verbatim; the earlier `total - keep` rows are replaced by a single summary row
- The summary row reads `| <first_date> — <last_date> | Compacted: <N> entries |` for 2-column tables, and inserts a `—` placeholder for the middle column(s) on 3+ column tables
- A generated summary row is recognized only when its first cell contains a non-empty `start — end` range, every interior cell is `—`, and its final cell has the exact grammatically-correct count plus `<!-- specsync:compact:v1 -->` provenance marker
- When new ordinary rows require another compaction, prior generated summary counts are accumulated and the original range start is retained
- Multiple marked summaries fail closed instead of being summed
- Column count and cells are parsed from the first contiguous table without treating odd-backslash escaped pipes or code-span pipes as delimiters; even backslash runs do not escape delimiters
- If `total <= keep`, no rows are removed (`removed == 0`) and the spec is not written; such results are filtered out by `compact_changelogs`
- `dry_run: true` collects `CompactResult` values but never writes files
- Only results where `removed > 0` are returned from `compact_changelogs`
- Re-running with no excess ordinary rows leaves the file byte-for-byte unchanged, including every LF/CRLF terminator
- `CompactResult.compacted_entries` reports ordinary entries retained and excludes the generated summary row
- Counts use fixed-width checked arithmetic and overflow fails closed
- All reads/parses and same-directory replacement staging complete before publication; preflight failure writes nothing
- The returned report identifies planned, succeeded, and failed operations so incomplete apply work cannot be reported as success

## Constraints

- Operates on the raw markdown text via string slicing; no markdown AST
- A spec is only touched if it contains a `## Change Log` marker; otherwise silently skipped
- Date range is taken from the first cell of the first/last removed rows (`extract_first_cell`)
- Command rendering reports structured failures and exits nonzero for every incomplete apply

## Out of Scope

- Compacting any section other than `## Change Log`
- Parsing or validating individual changelog dates
- Merging or de-duplicating ordinary changelog content beyond folding tool-generated summary rows
- Restoring previously compacted entries

### REQ-compact-001

The compact module SHALL compact only excess Change Log rows while preserving recent entries, table structure, and dry-run safety.

Acceptance Criteria
- `compact_changelogs(root, specs_dir, keep, dry_run)` walks every spec found by `find_spec_files` and compacts each spec's `## Change Log` table
- The `## Change Log` section ends at the next `## ` heading or EOF; only that slice is rewritten
- The first two `|`-prefixed lines in the section (header + separator) are always preserved
- The last `keep` ordinary data rows are kept verbatim; the earlier `total - keep` rows are replaced by a single summary row
- The summary row reads `| <first_date> — <last_date> | Compacted: <N> entries |` for 2-column tables, and inserts a `—` placeholder for the middle column(s) on 3+ column tables
- Column count is detected from the first data row (`| count - 1`)
- If `total <= keep`, no rows are removed (`removed == 0`) and the spec is not written; such results are filtered out by `compact_changelogs`
- `dry_run: true` collects `CompactResult` values but never writes files
- Only results where `removed > 0` are returned from `compact_changelogs`
