---
spec: cmd_compact.spec.md
---

## User Stories

- As a maintainer, I want old spec changelog entries trimmed so the `## Change Log` table in each spec stays readable and only keeps the most recent N rows
- As a maintainer, I want a `--dry-run` preview so I can see how many entries would be removed (and from which specs) before writing
- As a script author, I want a clear summary ("Compacted N entries across M spec(s)", or "No changelogs need compaction (all within limit).") so the result is easy to read or assert on

## Acceptance Criteria

- `cmd_compact(root, keep, dry_run)` loads config, resolves `config.specs_dir`, and delegates to `compact::compact_changelogs(root, &specs_dir, keep, dry_run)`
- `--keep N` controls how many changelog entries to retain per spec
- When `dry_run` is true, a banner prints, no files are written, and per-spec lines read "would compact"
- When `dry_run` is false, entries are removed and per-spec lines read "compacted"
- When the delegate returns no results, prints "No changelogs need compaction (all within limit)." and returns without a summary
- Each affected spec prints its relative `spec_path`, the `removed` count, and the kept (`compacted_entries`) count; the trailing summary sums removed entries across affected specs

## Constraints

- Pure orchestration wrapper: changelog parsing/trimming lives in `compact::compact_changelogs`; this module only loads config and formats output
- Must not panic on missing/unreadable specs — the underlying module skips them gracefully
- Output uses `colored` for status glyphs (`ℹ`, `✓`)

## Out of Scope

- The changelog-table parsing/rewriting logic (owned by the `compact` module)
- Compacting non-changelog content
- Interactive prompts or GUI
