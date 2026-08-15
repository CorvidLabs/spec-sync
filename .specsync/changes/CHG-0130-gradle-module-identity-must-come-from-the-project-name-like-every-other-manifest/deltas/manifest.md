## ADDED

### REQUIREMENT REQ-manifest-002

A Gradle project's module identity SHALL come from its project name.

Acceptance Criteria
- A single-project build is named from a literal `rootProject.name`.
- When `rootProject.name` is unset the project directory name is used, which is Gradle's own default rather than a spec-sync convention.
- A multi-project build continues to use its `include` names.
- No module name is derived from a source path segment, so neither the first nor the last segment of a package hierarchy can become a module.

## MODIFIED

### SPEC SECTION Public API

#### Exported Constants

| Constant | Type | Description |
|----------|------|-------------|
| `MAX_GRADLE_MANIFEST_BYTES` | `u64` | Crate-visible 4 MiB ceiling shared by retained Gradle manifest readers |

#### Exported Structs

| Struct | Fields | Description |
|--------|--------|-------------|
| `ManifestModule` | `name: String`, `source_paths: Vec<String>`, `dependencies: Vec<String>` | A module/target discovered from a manifest file |
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

