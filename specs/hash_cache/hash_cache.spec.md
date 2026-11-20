---
module: hash_cache
version: 11
status: stable
files:
  - src/hash_cache.rs
  - build.rs
db_tables: []
tracks: [90]
depends_on:
  - specs/parser/parser.spec.md
  - specs/util/util.spec.md
---

# Hash Cache

## Purpose

Uses SHA-256 content hashing to track which spec files, companion files, and source files have changed since the last validation run. Enables incremental validation — only specs with detected changes are re-validated, significantly improving performance on large projects.

## Public API

**Exported Constants**

| Name | Description |
|------|-------------|
| `BUILD_FINGERPRINT` | SHA-256 of `src/`, `Cargo.toml`, `Cargo.lock` and `build.rs` as this binary was compiled (from `build.rs`). The validation snapshot digest and the read-cache identity frame it, so neither cache crosses builds (REQ-hash-cache-005, REQ-change-134) |

**Exported Structs**

| Type | Description |
|------|-------------|
| `HashCache` | Persistent file hash storage — maps relative paths to hex SHA-256 digests. Stored as `hashes.json` in the project's local state folder outside the work tree (REQ-hash-cache-009) |
| `ChangeKind` | Enum classifying what changed: `Spec`, `Requirements`, `Companion`, `Source` |
| `ChangeClassification` | Result for one spec — contains `spec_path: PathBuf`, `changes: Vec<ChangeKind>`, and `baseline_known: bool` recording whether the cache held a prior hash |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load` | `root: &Path` | `Self` | HashCache: load `hashes.json` from the project's local state folder; returns an empty cache if it is missing, unparseable, a link, not a regular file or over the bound; never looks at `.specsync/hashes.json` in the work tree (REQ-hash-cache-007, REQ-hash-cache-009) |
| `save` | `&self, root: &Path` | `io::Result<()>` | HashCache: write pretty JSON to a new file in the local state folder and rename it over `hashes.json`; fails, writing nothing, when no local state folder can be found or opened (REQ-hash-cache-007, REQ-hash-cache-009) |
| `refusal` | `root: &Path` | `Option<String>` | HashCache: why this project's cache is not used: no local state folder, or what is at `hashes.json` there is a link, not a regular file or over the bound; `None` for a usable or absent cache |
| `cache_path` | `root: &Path` | `Option<PathBuf>` | HashCache: where the project's cache is kept, outside the work tree, for messages; `None` when no local state folder can be found |
| `hash_file` | `root: &Path, path: &Path` | `Option<String>` | HashCache: SHA-256 of a file read through the bounded project read (8 MiB); `None` when it is missing or refused, which the cache treats as changed (REQ-hash-cache-008) |
| `is_changed` | `&self, root: &Path, rel_path: &str` | `bool` | HashCache: true if file is new or hash differs from cached value |
| `has_baseline` | `&self, rel_path: &str` | `bool` | HashCache: whether a prior hash exists, so "changed" can be told apart from "never seen" |
| `update` | `&mut self, root: &Path, rel_path: &str` | `()` | HashCache: recompute and store hash for a file |
| `prune` | `&mut self, root: &Path` | `()` | HashCache: remove entries for files that no longer exist on disk |
| `has` | `&self, kind: ChangeKind` | `bool` | ChangeClassification: true if a specific change kind is present |
| `reportable` | `&self, kind: &ChangeKind` | `bool` | ChangeClassification: whether the change was observed against a real baseline and so is worth reporting rather than merely acting on |
| `classify_changes` | `root: &Path, spec_path: &Path, cache: &HashCache` | `ChangeClassification` | Check spec, companions, and source files for changes |
| `classify_all_changes` | `root: &Path, spec_files: &[PathBuf], cache: &HashCache` | `Vec<ChangeClassification>` | Classify all specs, returns only those with changes |
| `filter_unchanged` | `root: &Path, spec_files: &[PathBuf], cache: &HashCache` | `Vec<PathBuf>` | Return only specs with detected changes |
| `update_cache` | `root: &Path, spec_files: &[PathBuf], cache: &mut HashCache` | `()` | Post-validation: update hashes for all specs, companions, and source files; prune deleted entries |
| `extract_frontmatter_files` | `content: &str` | `Vec<String>` | Quick extraction of `files:` list from YAML frontmatter without full parser |
| `CachedValidationSnapshot` | Cached validation outcome bound to digests |
| `ValidationDiagnostics` | Cached errors/warnings/notices |
| `record_validation_snapshot` | Record warm-cache validation snapshot |
| `record_current_validation_snapshot` | Bind diagnostics to the current input digest and store them |
| `current_validation_input_digest` | Current validation input digest |
| `replayable_validation_snapshot` | Load replayable snapshot when still valid |
| `has_findings` | Whether a snapshot carries any error, warning, or notice |

## Invariants

1. Cache is stored as `hashes.json` in the project's local state folder (`util::local_state`), which is created automatically; never in the work tree
2. Missing or unparseable cache file is treated as empty cache (all files considered changed)
3. Unreadable files are treated as "changed" (conservative — triggers re-validation)
4. SHA-256 is computed in 8KB chunks for memory efficiency on large files
5. Path keys are normalized for cross-platform consistency (forward slashes)
6. Companion file detection covers all five companion types (requirements.md, context.md, tasks.md, testing.md, design.md) in both naming conventions: plain (`requirements.md`) and prefixed (`{module}.req.md`)
7. `update_cache` prunes entries for deleted files to prevent unbounded cache growth
8. `extract_frontmatter_files` uses quick string matching — does not invoke the full YAML parser
9. The validation snapshot input digest frames the name, type and content of every entry in the spec's `history/` directory, and the type of the directory itself, so a cached verdict never replays over an added, edited, removed or replaced history fragment, including one swapped for a symlink to the same bytes (REQ-hash-cache-006).
10. The cache is never read or written through a symbolic link. `load` and `save` work through the local state folder's handle, opened without following a link at any name SpecSync made; `save` creates a new file there exclusively and renames it over `hashes.json`, so the old file is never written in place, and a link found at that name is replaced, never followed. `.specsync` and `.specsync/hashes.json` in the work tree, which a repository can commit as links, are never looked at (REQ-hash-cache-007).
11. The cache is read within 64 MiB through the local state folder, any link refused, and every read of a project file here, every file it hashes or reads to classify a spec (8 MiB), goes through the bounded project read of `util`. A link that leaves the project, a dangling link, a file that is not regular and a file over its bound are refused and handled as that read's other read errors are, so nothing here waits on a FIFO or a device or reads past its bound (REQ-hash-cache-008).
12. A `.specsync/hashes.json` in the work tree, in any letter case and whether or not Git tracks it, came with the repository or with SpecSync 6.3 or earlier and is never read or saved over: Git's pathspecs are case-sensitive and an archive has no Git, so no check of it could be complete. No `git` process is started for the cache. `prune` keeps an entry only for a key `confine_path_to_root` accepts that names a regular project file, so it never looks at a path outside the project (REQ-hash-cache-009, REQ-hash-cache-010).

## Behavioral Examples

### Scenario: Incremental validation

- **Given** 50 specs, only 3 have changed since last run
- **When** `classify_all_changes` is called
- **Then** returns 3 `ChangeClassification` entries; 47 specs are skipped

### Scenario: Replay stored findings for an unchanged spec

- **Given** a snapshot recorded against the current spec, companions, sources, and global inputs
- **When** `replayable_validation_snapshot` is called
- **Then** it returns that snapshot, including any warnings the previous validation produced

### Scenario: Source file change triggers re-validation

- **Given** a spec lists `src/auth.rs` in frontmatter `files:`; that file has been modified
- **When** `classify_changes` is called for the spec
- **Then** returns `ChangeClassification` with `ChangeKind::Source` in changes

### Scenario: First run (no cache)

- **Given** no `hashes.json` in the project's local state folder
- **When** `HashCache::load` is called
- **Then** returns empty cache; all files will be classified as changed

### Scenario: A cloned repository commits the cache as a link

- **Given** `.specsync/hashes.json` is a symbolic link to a file outside the project
- **When** `HashCache::save` is called
- **Then** it writes the cache into the local state folder, the link stays, and the file it names keeps its bytes; `HashCache::load` never reads through the link and `HashCache::refusal` is `None`

### Scenario: Requirements change triggers staleness

- **Given** `requirements.md` companion has been updated
- **When** `classify_changes` is called for the parent spec
- **Then** returns `ChangeClassification` with `ChangeKind::Requirements`

### Scenario: Design or testing companion change detected

- **Given** `testing.md` or `design.md` has been modified
- **When** `classify_changes` is called for the parent spec
- **Then** returns `ChangeClassification` with `ChangeKind::Companion`

### Scenario: A history fragment replaced by a symlink or a directory re-validates

- **Given** a snapshot recorded while `specs/auth/history/add-passkeys.md` was a regular file
- **When** that fragment is replaced by a symlink to a copy of the same bytes, or by a directory of the same name, and `replayable_validation_snapshot` is called
- **Then** it returns `None`: the entry's type is part of the snapshot input digest, so `check` validates the spec again instead of replaying the earlier verdict

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cache file missing | Returns empty cache (all files treated as changed) |
| Cache file has invalid JSON | Returns empty cache silently |
| File unreadable during hashing | `hash_file` returns `None`; file treated as changed |
| No local state folder can be found or made | `load` returns an empty cache; `save` returns `io::Error`; `refusal` gives the reason |
| `.specsync` or `.specsync/hashes.json` in the work tree is a link, a directory or a file of any content | Never looked at: nothing is read, written or created through it |
| `hashes.json` in the local state folder is a symbolic link, a directory, or anything but a regular file | `load` returns an empty cache without reading it; `save` renames over a link and fails on a directory; `refusal` names the path |
| A project file this module reads leaves the project through a link, dangles, is not a regular file, or is over its bound | Refused without waiting or reading past the bound, and handled like that read's other read errors |
| `.specsync/hashes.json` is tracked by Git, or committed in another letter case | Never read or saved over; the cache in the local state folder is used as for any project |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| (external) | `sha2::Sha256` for content hashing |
| (external) | `serde` / `serde_json` for cache serialization |
| (external) | `cap-std` / `cap-primitives` for the no-follow `.specsync` directory handle the cache is read and written through |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `HashCache`, `classify_all_changes`, `update_cache` in `cmd_check` for incremental validation; `HashCache::refusal` for its one warning |

## Change Log

| Date | Change |
|------|--------|
| 2026-11-06 | #878: a validation snapshot replays only in the build that recorded it. `build.rs` compiles in a fingerprint of `src/`, `Cargo.toml`, `Cargo.lock` and `build.rs`, and the snapshot input digest frames it, so a clean result cached by an earlier build of the same version is re-validated (REQ-hash-cache-005). |
| 2026-10-10 | Add `hash_cache.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-04-13 | Document design.md and testing.md in companion file detection list |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document: Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally |
| 2026-08-15 | Persist and replay per-spec validation snapshots so a warm `check` cannot drop findings (#429) |
| 2026-08-16 | CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without: A warm hash cache must not drop findings, because skipping re-validation without replaying the previous result reports a passing spec that was never checked |
| 2026-11-06 | reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a: Reuse a cached validation snapshot or read-cache verdict only in the build that wrote it, so a stale pass cannot hide a new refusal (#878) |
| 2026-11-07 | run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds: Run clippy on every target in CI and the verify lane, and clear the test-code lints it finds |
