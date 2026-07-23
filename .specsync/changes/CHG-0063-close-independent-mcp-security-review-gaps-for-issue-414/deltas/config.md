## ADDED

### REQUIREMENT REQ-config-005

Configuration SHALL expose checked source-directory and manifest discovery that preserves malformed
or unreadable Gradle settings as errors while retaining infallible compatibility wrappers.

Acceptance Criteria

- Checked discovery returns an error before exposing partial manifest modules or source roots.
- `detect_source_dirs` remains compatible and falls back to scan-based discovery on a checked error.
- `discover_manifest_modules` remains compatible with its infallible discovery return type.
- Coverage and enforcement callers can use the checked variants to distinguish inconclusive
  discovery from successful empty discovery.

### REQUIREMENT REQ-config-006

Legacy JSON GitHub repository configuration SHALL fail closed when `github.repo` is present with a
non-string, non-null type.

Acceptance Criteria

- Number, boolean, object, and list values remain explicitly invalid instead of discarding the surrounding
  valid configuration or becoming repository auto-detection.
- Missing, null, and string repository values preserve compatibility.
- Issue inspection rejects the explicit invalid repository before no-spec/no-reference success.

### REQUIREMENT REQ-config-007

Configuration SHALL provide a checked parser for exact retained JSON/TOML bytes used by
security-sensitive callers.

Acceptance Criteria

- Parsing consumes supplied bytes without reopening the pathname.
- Leading BOM compatibility and omitted-source autodetection remain supported.
- Malformed syntax and wrong-shaped known TOML fields return an error instead of defaults.

## MODIFIED

### SPEC SECTION Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_config` | `root: &Path` | `SpecSyncConfig` | Load configuration in canonical-to-legacy precedence order, falling back to defaults with auto-detected source directories |
| `load_config_from_path` | `config_path: &Path, root: &Path` | `SpecSyncConfig` | Load config from a specific file path (JSON or TOML based on extension), used by migration |
| `detect_source_dirs` | `root: &Path` | `Vec<String>` | Compatibility source-directory discovery; falls back to scan-based detection when checked manifest discovery fails |
| `detect_source_dirs_checked` | `root: &Path` | `Result<Vec<String>, String>` | Auto-detect source directories while surfacing malformed or unreadable Gradle settings instead of returning partial manifest discovery |
| `default_schema_pattern` | — | `&'static str` | Returns the default regex for SQL CREATE TABLE extraction |
| `discover_manifest_modules` | `root: &Path` | `ManifestDiscovery` | Compatibility manifest discovery that preserves the infallible return type |
| `discover_manifest_modules_checked` | `root: &Path` | `Result<ManifestDiscovery, String>` | Discover manifest modules while surfacing malformed or unreadable Gradle settings |
| `is_legacy_layout` | `root: &Path` | `bool` | Detect whether a project uses a legacy 3.x layout (root-level config files without `.specsync/version` stamp) |
| `config_to_toml` | `config: &SpecSyncConfig` | `String` | Serialize a `SpecSyncConfig` to the current canonical `.specsync/config.toml` format |
| `config_to_toml_lossy_fields` | `config: &SpecSyncConfig` | `Vec<&'static str>` | List config fields `config_to_toml` cannot represent (e.g. `customRules`), so `migrate` can refuse rather than silently drop them |
| `read_config_file` | `path: &Path` | `Option<String>` | Read a config file, dropping a leading UTF-8 BOM (lossless) so it does not attach to the first TOML key or break JSON parsing; shared by the loaders and `migrate` so config reads handle a BOM consistently. `None` if unreadable |
| `parse_config_content_checked` | `config_path: &Path, content: &str, root: &Path` | `Result<SpecSyncConfig, String>` | Crate-private exact-byte JSON/TOML parser for retained callers; validates syntax and known TOML field types without reopening the path |
| `parse_config_content_checked_with_source_dirs` | `config_path: &Path, content: &str, root: &Path, detected_source_dirs: Option<Vec<String>>` | `Result<SpecSyncConfig, String>` | Crate-private exact-byte parser with caller-supplied retained source discovery for omitted source fields |

### SPEC SECTION Invariants

1. Config file search order is `.specsync/config.toml`, `.specsync/config.json`, `.specsync.toml`, `specsync.json`, then defaults.
2. When no config file exists, source directories are auto-detected from the project root.
3. When a config file exists but omits canonical `source_dirs` or legacy `sourceDirs`, source dirs are still auto-detected.
4. 46 common build/cache directories are always excluded from source detection.
5. `detect_source_dirs` falls back to `["src"]` if no source files are found.
6. Root-level source files produce `["."]` as source dirs.
7. TOML parsing is zero-dependency and uses line-by-line string parsing.
8. Basic and literal TOML strings preserve punctuation as content according to their string kind.
9. Present-but-unreadable config and local override files warn before built-in defaults are used; absent files apply defaults silently.
10. Retired AI key names are ignored with value-safe migration guidance and are never retained, serialized, printed, or executed.
11. Checked source-directory and manifest discovery fail before returning partial results when Gradle settings are malformed or unreadable; compatibility wrappers remain infallible for existing callers.
12. Checked retained-snapshot parsing validates real JSON/TOML syntax and known TOML field types
    before applying the established compatibility parser.
13. Capability callers may supply source-directory detection; omitted source fields consume that
    list without consulting an ambient root pathname.
