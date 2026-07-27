---
module: cmd_compact
version: 6
status: stable
files:
  - src/commands/compact.rs
db_tables: []
tracks: []
depends_on:
  - specs/compact/compact.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Compact

## Purpose

Implements the `specsync compact` command. Trims old entries from spec changelog sections, keeping only the most recent N entries.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_compact` | `root: &Path, keep: usize, dry_run: bool, format: OutputFormat` | `()` | Compact changelog entries and render text, JSON, or Markdown results |

## Invariants

1. Delegates to `compact::compact_changelogs()`
2. `--keep N` controls how many entries to retain (default 10)
3. Dry-run shows what would change without writing
4. Per-spec and aggregate output use correct singular/plural labels and exclude the generated summary from the kept count
5. JSON is one parseable, ANSI-free document; `--json` and `--format json` are equivalent
6. Markdown and GitHub formats render a heading, dry-run notice, result table, and truthful summary
7. Structured dry-run output distinguishes `would_change: true` from `applied: false`
8. JSON and Markdown project paths use `/` separators on Windows while preserving literal Unix backslashes
9. Markdown/GitHub paths use sanitized variable-length code spans that cannot inject table rows
10. JSON exposes `complete`, `partial`, planned/succeeded/failed operations, structured errors, and never sets `applied: true` for incomplete work
11. Any compact failure is rendered before the command exits 1

## Behavioral Examples

### Scenario: Compact changelogs

- **Given** a spec has 25 changelog entries, `--keep 10`
- **When** `cmd_compact` runs
- **Then** 15 oldest entries removed, 10 newest kept

### Scenario: Machine-readable dry run

- **Given** a spec has excess changelog entries
- **When** `cmd_compact` runs with JSON format and `dry_run = true`
- **Then** stdout is one valid JSON document with `would_change: true`, `applied: false`, and no file is modified

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No specs with changelogs | Prints "nothing to compact" |
| Fewer entries than keep | File unchanged |
| Any read, parse, stage, or publish failure | Renders a structured incomplete result and exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| compact | `compact_changelogs` |
| config | `load_config` |
| types | `OutputFormat` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync compact` |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | Harden #417 result truth: correct spec pluralization, platform-aware paths, safe Markdown code spans, structured failures, and nonzero incomplete outcomes |
| 2026-07-26 | Normalize structured result paths to portable `/` separators on Windows |
| 2026-07-26 | Fix #417 structured output: parse-clean JSON, Markdown/GitHub tables, shorthand equivalence, and explicit dry-run truth |
| 2026-07-26 | Fix #417 text reporting: truthful kept counts, correct singular/plural labels, and end-to-end dry-run/idempotence coverage |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
