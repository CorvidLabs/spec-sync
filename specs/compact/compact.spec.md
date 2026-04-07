---
module: compact
version: 1
status: stable
files:
  - src/compact.rs
db_tables: []
tracks: [94]
depends_on:
  - specs/validator/validator.spec.md
---

# Compact

## Purpose

Reduces changelog table size in spec files by keeping only the last N entries and summarizing older ones into a single compacted row with a date range and entry count.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `compact_changelogs` | `root: &Path, specs_dir: &Path, keep: usize, dry_run: bool` | `Vec<CompactResult>` | Compact changelog tables in all spec files, keeping the last `keep` entries |

### Exported Structs

| Type | Description |
|------|-------------|
| `CompactResult` | Result of compacting one spec — contains `spec_path: String`, `original_entries: usize`, `compacted_entries: usize`, `removed: usize` |

## Invariants

1. Only specs with a `## Change Log` section containing a markdown table are processed
2. Table header and separator rows (first two `|`-prefixed lines) are always preserved
3. The last `keep` data rows are preserved; earlier rows are summarized
4. Summary row contains the date range of compacted entries and their count
5. If a changelog has fewer than `keep + 1` entries, no compaction occurs
6. `dry_run: true` returns results without modifying files
7. Handles both 2-column and 3+ column tables with appropriate summary format

## Behavioral Examples

### Scenario: Compact a long changelog

- **Given** a spec with 20 changelog entries and `keep = 5`
- **When** `compact_changelogs` is called
- **Then** the first 15 entries are replaced with a single summary row like `| 2025-01-01 — 2026-03-15 | 15 entries compacted |`

### Scenario: Short changelog (no compaction needed)

- **Given** a spec with 3 changelog entries and `keep = 5`
- **When** `compact_changelogs` is called
- **Then** the spec is skipped (not included in results)

### Scenario: Dry run

- **Given** specs with long changelogs
- **When** `compact_changelogs(root, specs_dir, 5, true)` is called
- **Then** returns `CompactResult` entries but does not modify any files

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Prints error in bold red, continues processing other files |
| No changelog section found | Spec is silently skipped |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| validator | `find_spec_files` to locate all spec files |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `compact_changelogs` via `cmd_compact` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-06 | Initial spec for v3.3.0 |
