---
spec: cmd_compact.spec.md
---

## User Stories

- As a maintainer, I want old spec changelog entries trimmed so the `## Change Log` table in each spec stays readable and only keeps the most recent N rows
- As a maintainer, I want a `--dry-run` preview so I can see how many entries would be removed (and from which specs) before writing
- As a script author, I want a grammatically correct summary ("Compacted N entries across M specs", or "No changelogs need compaction (all within limit).") so the result is easy to read or assert on
- As an automation author, I want equivalent `--format json` and `--json` output so compaction results are safely parseable
- As a reviewer, I want `--format markdown` output with a result table suitable for a PR comment

## Acceptance Criteria

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
- Markdown/GitHub path cells use one sanitized code element (a variable-length Markdown span or entity-safe HTML code for literal-pipe paths) and cannot inject rows through pipes, backticks, controls, or bidi formatting
- JSON reports complete/partial state and planned/succeeded/failed operation counts; an incomplete apply never claims `applied: true`
- The command exits 1 after rendering any read, parse, stage, or publication failure

## Constraints

- Pure orchestration wrapper: changelog parsing/trimming lives in `compact::compact_changelogs`; this module only loads config and formats output
- Must not panic on missing/unreadable specs — the typed delegate report surfaces them as failures
- Text output uses `colored` for status glyphs (`ℹ`, `✓`); structured output contains no ANSI formatting

## Out of Scope

- The changelog-table parsing/rewriting logic (owned by the `compact` module)
- Compacting non-changelog content
- Interactive prompts or GUI

### REQ-cmd-compact-001

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

