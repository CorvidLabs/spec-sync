## MODIFIED

### REQUIREMENT REQ-manifest-021

Cargo discovery SHALL retain an explicitly declared library file as an exact source path of its package module, including workspace-relative paths. A project-root target SHALL be selected as that exact file rather than recursively scanning its parent project. Other automatic target-parent selections SHALL preserve whole-member boundaries for literal manifest-resolved workspace members; explicit directory selections retain their stated scope.

Acceptance Criteria
- A package demo with [lib].path="src/custom.rs" owns that exact file even though its stem is custom.
- A custom target outside src but inside the retained project remains measured under its declared owner without ambient probing; an escaping path remains an error.
- Root-level lib.rs/main.rs selections do not scan unrelated project files and retain checked exact-file identity.
- Targets declared by root packages or workspace members keep their exact owners without splitting ordinary or same-named files from another literal resolved member.
- Init, text, JSON and named batch generation retain these ownership and scan boundaries; existing files remain unchanged on rerun.
- Explicitly configured directories retain their authority. Workspace member glob expansion is not introduced.

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
| `ManifestDiscovery` | `modules: HashMap<String, ManifestModule>`, `source_dirs: Vec<String>`; crate-private `exact_cargo_source_files: HashSet<String>` | Aggregated checked discovery; scan selections are directories or exact declared Cargo files at the project root. |
| `GradleSettingsModule` | `name: String`, `path: String` | Crate-visible normalized Gradle module identity and effective project directory |

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `discover_from_manifests` | `root: &Path` | `ManifestDiscovery` | Compatibility discovery that returns an empty result when checked discovery is malformed |
| `discover_from_manifests_checked` | `root: &Path` | `Result<ManifestDiscovery, String>` | Discover modules while surfacing unreadable or malformed Gradle settings to gate callers |
| `discover_from_manifests_checked_with_root` | `root: &Path, project_root: &Dir` | `Result<ManifestDiscovery, String>` | Crate-visible checked discovery that reuses a caller-retained project-root capability and rejects an ambient/retained root identity mismatch |
| `parse_gradle_settings` | `content: &str` | `Result<Vec<GradleSettingsModule>, String>` | Crate-visible shared parser for Groovy/Kotlin includes plus assignment-style and method-style literal project-directory overrides |
| `is_jvm_package_source_root` | `source_dir: &str` | `bool` | Whether a directory is a JVM source root such as `src/main/kotlin`, whose children are package segments rather than modules. Naming a module from one of those segments is what made every `com.example.*` package collapse into a module called `com` (#473) |
