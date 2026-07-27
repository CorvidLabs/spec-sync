## MODIFIED

### REQUIREMENT REQ-cmd-compact-001

The compact command SHALL delegate deterministic changelog compaction and SHALL preserve dry-run and summary behavior.

Acceptance Criteria
- `cmd_compact(root, keep, dry_run, format)` loads config, resolves `config.specs_dir`, and delegates to `compact::compact_changelogs(root, &specs_dir, keep, dry_run)`
- `--keep N` controls how many changelog entries to retain per spec
- When `dry_run` is true, a banner prints, no files are written, and per-spec lines read "would compact"
- When `dry_run` is false, entries are removed and per-spec lines read "compacted"
- When the delegate returns no results, prints "No changelogs need compaction (all within limit)." and returns without a summary
- Each affected spec prints its relative `spec_path`, the `removed` count, and the kept (`compacted_entries`) count; the trailing summary sums removed entries across affected specs
- A count of one uses `entry`; all other counts use `entries`
- The kept count excludes the generated compaction summary row
- JSON mode emits one ANSI-free document with command, dry-run, `would_change`, `applied`, aggregate counts, and per-spec results
- In dry-run JSON, `would_change` reflects selected changes while `applied` remains false
- `--json` is byte-equivalent to `--format json`
- Markdown and GitHub modes emit a heading, optional dry-run notice, result table, and truthful singular/plural summary
- JSON and Markdown result paths use `/` separators on Windows while preserving literal Unix backslashes
- Markdown/GitHub paths are sanitized and rendered as one safe code element while preserving every legal Unix backslash parity
- JSON exposes complete/partial state, operation counts, and structured errors
- Any incomplete apply renders before exiting 1 and never claims `applied: true`

### SPEC SECTION Invariants

1. Delegates to `compact::compact_changelogs()`
2. `--keep N` controls how many entries to retain (default 10)
3. Dry-run shows what would change without writing
4. Per-spec and aggregate output use correct singular/plural labels and exclude the generated
   summary from the kept count.
5. JSON is one parseable, ANSI-free document; `--json` and `--format json` are equivalent.
6. Markdown and GitHub formats render a heading, dry-run notice, result table, and truthful summary.
7. Structured dry-run output distinguishes `would_change: true` from `applied: false`.
8. JSON and Markdown project paths use `/` separators on Windows while preserving literal Unix backslashes
9. Markdown/GitHub paths use one sanitized code element: variable-length Markdown spans normally and entity-safe HTML code when a literal pipe is present, so no table row can be injected and every legal Unix backslash is preserved
10. JSON exposes `complete`, `partial`, planned/succeeded/failed operations, structured errors, and never sets `applied: true` for incomplete work
11. Any compact failure is rendered before the command exits 1
