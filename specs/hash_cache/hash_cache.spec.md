---
module: hash_cache
version: 2
status: stable
files:
  - src/hash_cache.rs
db_tables: []
tracks: [90]
depends_on:
  - specs/parser/parser.spec.md
---

# Hash Cache

## Purpose

Uses SHA-256 content hashing to track which spec files, companion files, and source files have changed since the last validation run. Enables incremental validation — only specs with detected changes are re-validated, significantly improving performance on large projects.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `HashCache` | Persistent file hash storage — maps relative paths to hex SHA-256 digests. Stored in `.specsync/hashes.json` |
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
5. Path keys are normalized for cross-platform consistency (forward slashes)
6. Companion file detection covers all five companion types (requirements.md, context.md, tasks.md, testing.md, design.md) in both naming conventions: plain (`requirements.md`) and prefixed (`{module}.req.md`)
7. `update_cache` prunes entries for deleted files to prevent unbounded cache growth
8. `extract_frontmatter_files` uses quick string matching — does not invoke the full YAML parser

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
