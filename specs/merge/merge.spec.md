---
module: merge
version: 1
status: stable
files:
  - src/merge.rs
db_tables: []
tracks: [98]
depends_on:
  - specs/parser/parser.spec.md
  - specs/validator/validator.spec.md
---

# Merge

## Purpose

Detects and auto-resolves git merge conflicts in spec files using context-aware strategies. Different resolution algorithms are applied based on the section type — YAML frontmatter fields are unioned, changelog tables are merged chronologically, and generic tables are deduplicated by first cell.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `merge_specs` | `root: &Path, specs_dir: &Path, dry_run: bool, all_files: bool` | `Vec<MergeResult>` | Scan for conflicted specs and attempt auto-resolution |
| `has_conflict_markers` | `content: &str` | `bool` | Check if content contains `<<<<<<< ` conflict markers |
| `print_results` | `results: &[MergeResult], dry_run: bool` | `()` | Print human-readable resolution summary with colored output |
| `results_to_json` | `results: &[MergeResult]` | `String` | Format results as JSON with path, status, and details |

### Exported Structs/Enums

| Type | Description |
|------|-------------|
| `MergeResult` | Outcome for one spec — `spec_path: String`, `status: MergeStatus`, `details: Vec<String>` |
| `MergeStatus` | Enum: `Resolved` (auto-resolved), `Manual` (needs human), `Clean` (no conflicts) |

### Resolution Strategies

| Context | Strategy |
|---------|----------|
| Frontmatter (YAML) | Lists (`files`, `db_tables`, `depends_on`) are unioned and sorted; scalars use theirs (latest) |
| `## Change Log` table | Rows merged chronologically by date cell; deduplicated by full row content |
| Generic markdown table | Rows merged by first cell (symbol name); theirs wins on conflicts; deduplicated |
| Prose / section body | No auto-resolution — conflict markers preserved for manual intervention |

## Invariants

1. `all_files: false` uses `git diff --diff-filter=U` to find only git-conflicted files
2. `all_files: true` scans all spec files for conflict markers regardless of git state
3. Frontmatter list fields are unioned (not replaced) and sorted alphabetically
4. Frontmatter scalar fields use "theirs wins" strategy (latest change takes precedence)
5. Changelog rows are sorted by first cell (ISO date) — lexicographic sorting works correctly
6. Prose conflicts are never auto-resolved — always marked as `Manual`
7. Post-resolution, frontmatter is re-validated; warnings printed if invalid
8. `dry_run: true` returns results without writing resolved content to disk
9. Custom YAML parser handles simple key-value and list fields without external YAML library

## Behavioral Examples

### Scenario: Auto-resolve frontmatter list conflict

- **Given** ours has `files: [a.rs, b.rs]` and theirs has `files: [b.rs, c.rs]`
- **When** `merge_specs` resolves the conflict
- **Then** result is `files: [a.rs, b.rs, c.rs]` (union, sorted)

### Scenario: Auto-resolve changelog conflict

- **Given** ours added a `2026-03-20` entry, theirs added a `2026-03-25` entry
- **When** `merge_specs` resolves the conflict
- **Then** both entries appear in chronological order, status is `Resolved`

### Scenario: Prose conflict requires manual resolution

- **Given** both sides modified the `## Purpose` section text
- **When** `merge_specs` encounters the conflict
- **Then** conflict markers are preserved, status is `Manual`

### Scenario: Dry run

- **Given** conflicted spec files exist
- **When** `merge_specs(root, specs_dir, true, false)` is called
- **Then** returns `MergeResult` entries with resolution details but does not modify files

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Marked as `Manual` with read error in details |
| `git diff` command fails | Falls back to scanning all files for conflict markers |
| Post-resolution frontmatter invalid | Warning printed; file is still written with resolved content |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter` for post-resolution validation |
| validator | `find_spec_files` to locate all spec files when `all_files: true` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `merge_specs`, `print_results`, `results_to_json` via `cmd_merge` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-06 | Initial spec for v3.3.0 |
