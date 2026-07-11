---
module: cmd_compact
version: 1
status: stable
files:
  - src/commands/compact.rs
db_tables: []
tracks: []
depends_on:
  - specs/compact/compact.spec.md
  - specs/config/config.spec.md
---

# Cmd Compact

## Purpose

Implements the `specsync compact` command. Trims old entries from spec changelog sections, keeping only the most recent N entries.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_compact` | `root: &Path, keep: usize, dry_run: bool` | `()` | Compact changelog entries across all spec files |

## Invariants

1. Delegates to `compact::compact_changelogs()`
2. `--keep N` controls how many entries to retain (default 10)
3. Dry-run shows what would change without writing

## Behavioral Examples

### Scenario: Compact changelogs

- **Given** a spec has 25 changelog entries, `--keep 10`
- **When** `cmd_compact` runs
- **Then** 15 oldest entries removed, 10 newest kept

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No specs with changelogs | Prints "nothing to compact" |
| Fewer entries than keep | File unchanged |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| compact | `compact_changelogs` |
| config | `load_config` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync compact` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
