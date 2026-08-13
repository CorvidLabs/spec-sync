## ADDED

### REQUIREMENT REQ-hash-cache-002

A change classification SHALL record whether the cache held a prior hash for the spec, so
that a change can be told apart from an absence of evidence.

Acceptance Criteria
- An absent cache entry continues to select the spec for re-validation.
- An absent cache entry is not reported as a change.
- A companion change observed against a known baseline is still reported.
- The cache's own frontmatter `files:` extraction resolves a quoted entry to the same path the parser resolves.

## MODIFIED

### SPEC SECTION Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `HashCache` | Persistent file hash storage — maps relative paths to hex SHA-256 digests. Stored in `.specsync/hashes.json` |
| `ChangeKind` | Enum classifying what changed: `Spec`, `Requirements`, `Companion`, `Source` |
| `ChangeClassification` | Result for one spec — contains `spec_path: PathBuf`, `changes: Vec<ChangeKind>`, and `baseline_known: bool` recording whether the cache held a prior hash |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load` | `root: &Path` | `Self` | HashCache: load from `.specsync/hashes.json`; returns empty cache if missing |
| `save` | `&self, root: &Path` | `io::Result<()>` | HashCache: write to `.specsync/hashes.json` with pretty JSON |
| `hash_file` | `path: &Path` | `Option<String>` | HashCache: compute SHA-256 of file in 8KB chunks; returns hex string |
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
| `current_validation_input_digest` | Current validation input digest |
| `replayable_validation_snapshot` | Load replayable snapshot when still valid |

