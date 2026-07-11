## ADDED

### REQUIREMENT REQ-cmd-compact-001

The compact command SHALL delegate deterministic changelog compaction and SHALL preserve dry-run and summary behavior.

Acceptance Criteria
- `cmd_compact(root, keep, dry_run)` loads config, resolves `config.specs_dir`, and delegates to `compact::compact_changelogs(root, &specs_dir, keep, dry_run)`
- `--keep N` controls how many changelog entries to retain per spec
- When `dry_run` is true, a banner prints, no files are written, and per-spec lines read "would compact"
- When `dry_run` is false, entries are removed and per-spec lines read "compacted"
- When the delegate returns no results, prints "No changelogs need compaction (all within limit)." and returns without a summary
- Each affected spec prints its relative `spec_path`, the `removed` count, and the kept (`compacted_entries`) count; the trailing summary sums removed entries across affected specs
