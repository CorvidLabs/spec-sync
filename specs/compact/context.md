---
spec: compact.spec.md
---

## Key Decisions

- **String slicing, not markdown parsing**: `compact_spec_changelog` finds the `## Change Log` marker, computes the section end (next `## ` heading or EOF), and rewrites only that slice. The rest of the file is copied byte-for-byte.
- **First two table lines are structural**: any `|`-prefixed line counts toward the table; the first two are treated as header + separator and always kept, the rest are data rows.
- **Keep-last semantics**: the most recent `keep` rows survive; older rows collapse into one summary row inserted at the position of the first removed row.
- **Column-aware summary**: column count comes from the first data row; 3+ column tables get a `—` placeholder for the interior column so the markdown table stays aligned.
- **No-op filtering**: `compact_spec_changelog` still returns a `CompactResult` when nothing is removed, but `compact_changelogs` drops those (only `removed > 0` is surfaced and written).

## Files to Read First

- `src/compact.rs` — entire module: `compact_changelogs` (driver), `compact_spec_changelog` (core rewrite), `extract_first_cell`, and the `CompactResult` struct.

## Current Status

Stable and complete. Public API is `compact_changelogs` plus the `CompactResult` struct. Invoked by the `cmd_compact` subcommand.

## Notes

- Depends on `validator::find_spec_files`.
- `dry_run` short-circuits the `fs::write` only; results are still collected.
- Summary row format: `| <first> — <last> | Compacted: <N> entries |` (with an extra `—` cell for wider tables).
