---
module: cmd_merge
version: 1
status: stable
files:
  - src/commands/merge.rs
db_tables: []
tracks: []
depends_on:
  - specs/merge/merge.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Merge

## Purpose

Implements the `specsync merge` command. Auto-resolves git merge conflicts in spec files, flagging files that need manual resolution.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_merge` | `root: &Path, dry_run: bool, all: bool, format: OutputFormat` | `()` | Resolve merge conflicts in spec files |

## Invariants

1. Delegates to `merge::merge_specs()`
2. `--all` scans all spec files for conflict markers
3. Exits 1 if any merge needs manual resolution

## Behavioral Examples

### Scenario: Auto-resolved

- **Given** 3 specs with simple conflicts
- **When** `cmd_merge` runs
- **Then** all auto-resolved

### Scenario: Manual needed

- **Given** complex conflict
- **When** `cmd_merge` runs
- **Then** flags file, exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No conflicts | Prints "no conflicts" |
| Complex conflict | Exits 1 with file path |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| merge | `merge_specs`, `MergeStatus` |
| config | `load_config` |
| types | `OutputFormat` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync merge` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
