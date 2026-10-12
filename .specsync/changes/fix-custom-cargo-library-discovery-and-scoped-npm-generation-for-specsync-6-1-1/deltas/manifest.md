## ADDED

### REQUIREMENT REQ-manifest-021

Cargo discovery SHALL retain an explicitly declared library file as an exact source path of its package module and add its parent to scan directories, including workspace-relative paths.

Acceptance Criteria
- A package demo with [lib].path="src/custom.rs" owns that exact file even though its stem is custom.
- A path outside src is measured through its declared parent without ambient probing.
- Package source roots, binary targets and workspace prefixes keep their existing ownership.

## MODIFIED

### SPEC SECTION Public API

#### Exported Constants

| Constant | Type | Description |
|----------|------|-------------|
| `MAX_GRADLE_MANIFEST_BYTES` | `u64` | Crate-visible 4 MiB ceiling shared by retained Gradle manifest readers |

#### Exported Structs

| Struct | Fields | Description |
|--------|--------|-------------|
| `ManifestModule` | `name: String`, `source_paths: Vec<String>`, `dependencies: Vec<String>` | A module/target discovered from a manifest file; source paths are directories or exact Cargo library/binary files |
| `ManifestDiscovery` | `modules: HashMap<String, ManifestModule>`, `source_dirs: Vec<String>` | Aggregated result of parsing all manifest files in a project |
| `GradleSettingsModule` | `name: String`, `path: String` | Crate-visible normalized Gradle module identity and effective project directory |

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `discover_from_manifests` | `root: &Path` | `ManifestDiscovery` | Compatibility discovery that returns an empty result when checked discovery is malformed |
| `discover_from_manifests_checked` | `root: &Path` | `Result<ManifestDiscovery, String>` | Discover modules while surfacing unreadable or malformed Gradle settings to gate callers |
| `discover_from_manifests_checked_with_root` | `root: &Path, project_root: &Dir` | `Result<ManifestDiscovery, String>` | Crate-visible checked discovery that reuses a caller-retained project-root capability and rejects an ambient/retained root identity mismatch |
| `parse_gradle_settings` | `content: &str` | `Result<Vec<GradleSettingsModule>, String>` | Crate-visible shared parser for Groovy/Kotlin includes plus assignment-style and method-style literal project-directory overrides |
| `is_jvm_package_source_root` | `source_dir: &str` | `bool` | Whether a directory is a JVM source root such as `src/main/kotlin`, whose children are package segments rather than modules. Naming a module from one of those segments is what made every `com.example.*` package collapse into a module called `com` (#473) |
