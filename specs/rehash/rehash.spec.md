---
module: rehash
version: 2
status: stable
files:
  - src/commands/rehash.rs
db_tables: []
tracks: []
depends_on:
  - specs/hash_cache/hash_cache.spec.md
  - specs/config/config.spec.md
  - specs/validator/validator.spec.md
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

1. Loads canonical configuration and discovers spec files through `config::load_config` and
   `validator::find_spec_files`
2. Filters underscore-prefixed template specs and exits successfully with generation guidance when no specs exist
3. Builds a fresh `HashCache` from scratch (not incremental)
4. Saves cache to `.specsync/hashes.json`
5. Exits with code 1 if cache save fails

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

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| validator | `find_spec_files` |
| hash_cache | `HashCache::default`, `update_cache`, `save` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-11 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
