## ADDED

### REQUIREMENT REQ-config-010

The default configuration loader SHALL refuse an unloadable config file.

Acceptance Criteria
- A config file that exists and cannot be used stops the command, whichever loader it reached.
- A permissive loader remains available under a name that states it bypasses the refusal, so the bypass is deliberate rather than forgotten.
- A project with no config file at all is unaffected; the built-in defaults remain a legitimate run.

## MODIFIED

### SPEC SECTION Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_config` | `root: &Path` | `SpecSyncConfig` | Load configuration in canonical-to-legacy precedence order, refusing a config file that exists and cannot be used; absent config falls back to defaults with auto-detected source directories |
| `load_config_allowing_unloadable` | `root: &Path` | `SpecSyncConfig` | Like `load_config` but does NOT refuse a config file that exists and cannot be used. Only for callers whose job is repairing a broken config, which must run on the project that needs repairing. Named for the bypass so that omitting the guard is impossible and choosing to skip it is deliberate |
| `load_config_from_path` | `config_path: &Path, root: &Path` | `SpecSyncConfig` | Load config from a specific file path (JSON or TOML based on extension), used by migration |
| `detect_source_dirs` | `root: &Path` | `Vec<String>` | Compatibility source-directory discovery; falls back to scan-based detection when checked manifest discovery fails |
| `detect_source_dirs_checked` | `root: &Path` | `Result<Vec<String>, String>` | Auto-detect source directories while surfacing malformed or unreadable Gradle settings instead of returning partial manifest discovery |
| `source_detection_ignores_directory` | `name: &str` | `bool` | Crate-visible shared classification for hidden and configured source-detection ignore names |
| `default_schema_pattern` | — | `&'static str` | Returns the default regex for SQL CREATE TABLE extraction |
| `discover_manifest_modules` | `root: &Path` | `ManifestDiscovery` | Compatibility manifest discovery that preserves the infallible return type |
| `discover_manifest_modules_checked` | `root: &Path` | `Result<ManifestDiscovery, String>` | Discover manifest modules while surfacing malformed or unreadable Gradle settings |
| `is_legacy_layout` | `root: &Path` | `bool` | Detect whether a project uses a legacy 3.x layout (root-level config files without `.specsync/version` stamp) |
| `config_to_toml` | `config: &SpecSyncConfig` | `String` | Serialize a `SpecSyncConfig` to the current canonical `.specsync/config.toml` format |
| `config_to_toml_lossy_fields` | `config: &SpecSyncConfig` | `Vec<&'static str>` | List config fields `config_to_toml` cannot represent (e.g. `customRules`), so `migrate` can refuse rather than silently drop them |
| `read_config_file` | `path: &Path` | `Option<String>` | Read a config file, dropping a leading UTF-8 BOM (lossless) so it does not attach to the first TOML key or break JSON parsing; shared by the loaders and `migrate` so config reads handle a BOM consistently. `None` if unreadable |
| `parse_config_content_checked` | `config_path: &Path, content: &str, root: &Path` | `Result<SpecSyncConfig, String>` | Crate-private exact-byte JSON/TOML parser for retained callers; validates syntax and known TOML field types without reopening the path |
| `parse_config_content_checked_with_source_dirs` | `config_path: &Path, content: &str, root: &Path, detected_source_dirs: Option<Vec<String>>` | `Result<SpecSyncConfig, String>` | Crate-private exact-byte parser with caller-supplied retained source discovery for omitted source fields |
| `is_detectable_source_file` | `path: &Path` | `bool` | Crate-private lexical classifier shared by ambient and retained source detection |

| Constant | Type | Description |
|----------|------|-------------|
| `CONFIG_PATH_CANDIDATES` | `&[&str]` | Crate-private canonical-to-legacy configuration precedence shared by retained CLI discovery |
| `detect_source_dirs_with_confidence` | Detect source dirs with confidence flag |
| `validate_config_file` | Validate config file honestly |

