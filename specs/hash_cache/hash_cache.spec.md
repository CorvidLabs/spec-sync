---
module: hash_cache
version: 4
status: stable
files:
  - src/hash_cache.rs
db_tables: []
tracks: [90, 429]
depends_on:
  - specs/parser/parser.spec.md
---

# Hash Cache

## Purpose

Uses SHA-256 content hashing to track which validation inputs changed and stores versioned, integrity-checked per-spec validation snapshots. Incremental checks may replay a snapshot only when the exact spec, companion, source, global, and spec-inventory inputs still match.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `HashCache` | Persistent file hash storage — maps relative paths to hex SHA-256 digests. Stored in `.specsync/hashes.json` |
| `CachedValidationSnapshot` | Versioned platform-native display path, errors, warnings, notices, input digest, and integrity digest for one validated spec |
| `ValidationDiagnostics` | Errors, unsuppressed warnings, and notices captured from one shared validation result before snapshot binding |
| `ChangeKind` | Enum classifying what changed: `Spec`, `Requirements`, `Companion`, `Source` |
| `ChangeClassification` | Result for one spec — contains `spec_path: PathBuf` and `changes: Vec<ChangeKind>` |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load` | `root: &Path` | `Self` | HashCache: load from `.specsync/hashes.json`; returns empty cache if missing |
| `save` | `&self, root: &Path` | `io::Result<()>` | HashCache: write to `.specsync/hashes.json` with pretty JSON |
| `hash_file` | `path: &Path` | `Option<String>` | HashCache: compute SHA-256 of file in 8KB chunks; returns hex string |
| `is_changed` | `&self, root: &Path, rel_path: &str` | `bool` | HashCache: true if file is new or hash differs from cached value |
| `update` | `&mut self, root: &Path, rel_path: &str` | `()` | HashCache: recompute and store hash for a file |
| `prune` | `&mut self, root: &Path` | `()` | HashCache: remove entries for files that no longer exist on disk |
| `record_validation_snapshot` | `&mut self, root, spec_path, global_inputs, spec_inventory, expected_input_digest, diagnostics` | `bool` | Bind diagnostics only when inputs still match the digest captured immediately before validation; otherwise remove stale state and return false |
| `current_validation_input_digest` | `root, spec_path, global_inputs, spec_inventory` | `String` | Compute the deterministic digest used before and after validation to prevent race-bound stale snapshots |
| `replayable_validation_snapshot` | `&self, root, spec_path, global_inputs, spec_inventory` | `Option<&CachedValidationSnapshot>` | Return a snapshot only when format, snapshot version, integrity, and current input digest all match |
| `has` | `&self, kind: ChangeKind` | `bool` | ChangeClassification: true if a specific change kind is present |
| `classify_changes` | `root: &Path, spec_path: &Path, cache: &HashCache` | `ChangeClassification` | Check spec, companions, and source files for changes |
| `classify_all_changes` | `root: &Path, spec_files: &[PathBuf], cache: &HashCache` | `Vec<ChangeClassification>` | Classify all specs, returns only those with changes |
| `filter_unchanged` | `root: &Path, spec_files: &[PathBuf], cache: &HashCache` | `Vec<PathBuf>` | Return only specs with detected changes |
| `update_cache` | `root: &Path, spec_files: &[PathBuf], cache: &mut HashCache` | `()` | Post-validation: update hashes for all specs, companions, and source files; prune deleted entries |
| `extract_frontmatter_files` | `content: &str` | `Vec<String>` | Quick extraction of `files:` list from YAML frontmatter without full parser |

## Invariants

1. Cache is stored at `{root}/.specsync/hashes.json`; the `.specsync/` directory is created automatically
2. Missing or unparseable cache file is treated as empty cache (all files considered changed)
3. Unreadable files are treated as "changed" (conservative — triggers re-validation)
4. SHA-256 is computed in 8KB chunks for memory efficiency on large files
5. Path keys are normalized for cross-platform consistency (forward slashes) and serialized from ordered maps for deterministic cache bytes
6. Companion file detection covers all five companion types (requirements.md, context.md, tasks.md, testing.md, design.md) in both naming conventions: plain (`requirements.md`) and prefixed (`{module}.req.md`)
7. `update_cache` prunes entries for deleted files to prevent unbounded cache growth
8. `extract_frontmatter_files` uses quick string matching — does not invoke the full YAML parser
9. Cache format and snapshot schemas are explicitly versioned; missing, malformed, unknown-version, or unknown-shape cache data is invalidated and causes validation rather than a skip
10. Snapshot input digests bind the spec, existing companions, frontmatter source files, recursive schema/config/ignore inputs, package version, and sorted complete spec inventory
11. Snapshot integrity digests cover the platform-native cold-output path and every diagnostic field so partial or tampered outcomes cannot produce a false-green replay
12. Snapshot recording compares input digests from immediately before and after validation; a concurrent input change prevents the snapshot from being stored

## Behavioral Examples

### Scenario: Incremental validation

- **Given** 50 specs, only 3 have changed since last run
- **When** `classify_all_changes` is called
- **Then** returns 3 `ChangeClassification` entries; 47 specs are skipped

### Scenario: Source file change triggers re-validation

- **Given** a spec lists `src/auth.rs` in frontmatter `files:`; that file has been modified
- **When** `classify_changes` is called for the spec
- **Then** returns `ChangeClassification` with `ChangeKind::Source` in changes

### Scenario: First run (no cache)

- **Given** `.specsync/hashes.json` does not exist
- **When** `HashCache::load` is called
- **Then** returns empty cache; all files will be classified as changed

### Scenario: Warm validation replay

- **Given** a version-compatible snapshot whose integrity and exact input digest still match
- **When** the check command asks for a replayable snapshot
- **Then** the cached errors, warnings, and notices are returned without re-validating the spec

### Scenario: Stale or incompatible snapshot

- **Given** a source, ignore rule, schema file, or spec inventory changed, or the cache/snapshot version is unknown
- **When** the check command asks for a replayable snapshot
- **Then** no snapshot is returned and the spec is re-validated

### Scenario: Requirements change triggers staleness

- **Given** `requirements.md` companion has been updated
- **When** `classify_changes` is called for the parent spec
- **Then** returns `ChangeClassification` with `ChangeKind::Requirements`

### Scenario: Design or testing companion change detected

- **Given** `testing.md` or `design.md` has been modified
- **When** `classify_changes` is called for the parent spec
- **Then** returns `ChangeClassification` with `ChangeKind::Companion`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cache file missing | Returns empty cache (all files treated as changed) |
| Cache file has invalid JSON | Returns empty cache silently |
| Cache or snapshot version is unsupported | Invalidates the incompatible state and forces validation |
| Snapshot digest or current input digest does not match | Refuses replay and forces validation |
| File unreadable during hashing | `hash_file` returns `None`; file treated as changed |
| Cannot create `.specsync/` directory | `save` returns `io::Error` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| (external) | `sha2::Sha256` for content hashing |
| (external) | `serde` / `serde_json` for cache serialization |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `HashCache`, `classify_all_changes`, `update_cache` in `cmd_check` for incremental validation |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-04-13 | Document design.md and testing.md in companion file detection list |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-26 | v4: persist complete versioned validation snapshots and reject malformed, stale, incompatible, or integrity-mismatched replay state for issue #429 |
