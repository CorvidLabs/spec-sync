---
module: rehash
version: 1
status: stable
files:
  - src/commands/rehash.rs
db_tables: []
tracks: []
depends_on:
  - specs/hash_cache/hash_cache.spec.md
  - specs/commands/commands.spec.md
---

# Rehash

## Purpose

Implements the `specsync rehash` command. Regenerates the `.specsync/hashes.json` cache for all discovered spec files. Useful after `git pull`, branch switches, or when hashes.json is gitignored and needs rebuilding.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_rehash` | `root: &Path` | `()` | Discover all specs and regenerate the hash cache file |

## Invariants

1. Loads config and discovers specs via `load_and_discover`
2. Builds a fresh `HashCache` from scratch (not incremental)
3. Saves cache to `.specsync/hashes.json`
4. Exits with code 1 if cache save fails

## Behavioral Examples

### Scenario: Normal rehash

- **Given** a valid specsync project with specs
- **When** `cmd_rehash(root)` runs
- **Then** writes fresh hashes.json and prints spec count

### Scenario: Save failure

- **Given** .specsync directory is not writable
- **When** `cmd_rehash(root)` runs
- **Then** prints error and exits with code 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cache save fails | Prints error, exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| hash_cache | `HashCache`, `update_cache` |
| commands | `load_and_discover` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync rehash` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-11 | Initial spec |
