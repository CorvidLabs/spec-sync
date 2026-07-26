---
module: rehash
version: 3
status: stable
files:
  - src/commands/rehash.rs
db_tables: []
tracks: [429]
depends_on:
  - specs/hash_cache/hash_cache.spec.md
  - specs/config/config.spec.md
  - specs/ignore/ignore.spec.md
  - specs/validator/validator.spec.md
---

# Rehash

## Purpose

Implements the `specsync rehash` command. Regenerates `.specsync/hashes.json` with current file hashes, global validation inputs, and complete per-spec validation snapshots so the first unchanged check can safely use a warm cache.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_rehash` | `root: &Path` | `()` | Discover all specs and regenerate hashes, global inputs, and safe validation snapshots |

## Invariants

1. Loads canonical configuration and discovers spec files through `config::load_config` and
   `validator::find_spec_files`
2. Filters underscore-prefixed template specs and exits successfully with generation guidance when no specs exist
3. Builds a fresh `HashCache` from scratch (not incremental)
4. Runs the shared collected validation path for every discovered spec; records complete input-bound snapshots only when the project has no validation errors, without changing rehash's own exit gate
5. Hashes config, recursive schema files, and `.specsyncignore` alongside per-spec inputs
6. Saves cache to `.specsync/hashes.json`
7. Exits with code 1 if cache save fails

## Behavioral Examples

### Scenario: Normal rehash

- **Given** a valid specsync project with specs
- **When** `cmd_rehash(root)` runs
- **Then** writes fresh hashes and validation snapshots, then prints the spec count

### Scenario: First check after rehash

- **Given** rehash completed and no validation input changed
- **When** non-strict `specsync check --format json` runs
- **Then** it reports the complete cached findings with `specs_cached` equal to the full spec count

### Scenario: Save failure

- **Given** .specsync directory is not writable
- **When** `cmd_rehash(root)` runs
- **Then** prints error and exits with code 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cache save fails | Prints error, exits 1 |
| Validation reports any error during rehash | Writes current hashes but clears replayable snapshots so the next check performs full validation and reports the errors |

## Dependencies

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| validator | `find_spec_files` |
| hash_cache | `HashCache::default`, `update_cache`, `save` |
| ignore | `IgnoreRules::load` for the same filtered warning outcome as check |
| commands | Shared schema, global-input, inventory, and snapshot-aware validation helpers |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-11 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-26 | v3: rebuild complete versioned validation snapshots and global-input hashes for issue #429 |
