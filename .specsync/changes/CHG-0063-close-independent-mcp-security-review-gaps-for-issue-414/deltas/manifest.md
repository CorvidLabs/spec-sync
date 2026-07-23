## MODIFIED

### REQUIREMENT REQ-manifest-001

Manifest discovery SHALL identify supported project modules and source roots deterministically
without claiming unsupported workspace expansion.

Acceptance Criteria

- `discover_from_manifests_checked` surfaces malformed Gradle settings to coverage and generation
  gates rather than returning partial discovery, and merges the exact parse from that same read
  instead of validating then rereading a mutable path.
- One parser handles Groovy/Kotlin comments, escapes, literal multiline includes, nested colon
  names, and exactly `file(<literal>)` or `new File(rootDir, <literal>)` project directories.
- Dynamic include arguments, alternate `new File` bases, extra arguments, and trailing assignment
  expressions fail checked discovery without returning partial modules.
- Gradle module identities and `projectDir` literals are confined to project-relative paths:
  rooted, drive-qualified, UNC, and parent-underflow forms fail before source probing, while safe
  literal spellings retain compatibility.
- General module discovery and MCP snapshot preflight use the same effective Gradle module paths.
- A present `settings.gradle[.kts]` is parsed and validated even when no root
  `build.gradle[.kts]` exists.
- MCP Cargo workspace snapshot and confinement discovery parse bounded manifests as real TOML.
- Malformed MCP Cargo TOML/workspace shapes make MCP operations inconclusive; malformed Gradle
  declarations make checked coverage gates inconclusive without partial module results.

### SPEC SECTION Public API

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
| `parse_gradle_settings` | `content: &str` | `Result<Vec<GradleSettingsModule>, String>` | Crate-visible shared parser for Groovy/Kotlin includes and project-directory overrides |

### SPEC SECTION Invariants

1. Gradle settings parsing is comment- and escape-aware and supports literal Groovy/Kotlin multiline
   include declarations.
2. Nested colon names and the supported literal `projectDir` forms resolve to one deterministic
   effective project-relative directory per module.
3. Dynamic includes and unsupported `projectDir` bases, arity, or suffixes fail without partial
   discovery.
4. Checked discovery reports malformed Gradle input; compatibility discovery may return an empty
   result, but gate callers remain inconclusive.
5. Checked Gradle discovery merges the exact single-read parse result.
6. MCP Cargo workspace discovery trusts only structurally parsed TOML members and target paths.
7. Settings-only Gradle workspaces discover included modules, while malformed settings remain an
   inconclusive checked-discovery error.
8. Gradle module and effective project-directory paths cannot traverse above the project root or
   select rooted, drive-qualified, or UNC locations; rejection occurs before partial discovery or
   filesystem probing.
