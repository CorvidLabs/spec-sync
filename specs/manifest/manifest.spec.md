---
module: manifest
version: 6
status: stable
files:
  - src/manifest.rs
db_tables: []
tracks: [55]
depends_on: []
---

# Manifest

## Purpose

Manifest-aware module detection for multi-language projects. Parses language-specific manifest/build files (Cargo.toml, Package.swift, build.gradle.kts, package.json, pubspec.yaml, go.mod, pyproject.toml) to discover targets, source paths, module names, and dependencies — replacing pure directory scanning with structured project metadata.

## Public API

### Exported Structs

| Struct | Fields | Description |
|--------|--------|-------------|
| `ManifestModule` | `name: String`, `source_paths: Vec<String>`, `dependencies: Vec<String>` | A module/target discovered from a manifest file |
| `ManifestDiscovery` | `modules: HashMap<String, ManifestModule>`, `source_dirs: Vec<String>` | Aggregated result of parsing all manifest files in a project |
| `GradleSettingsModule` | `name: String`, `path: String` | Crate-visible normalized Gradle module identity and effective project directory |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `discover_from_manifests` | `root: &Path` | `ManifestDiscovery` | Compatibility discovery that returns an empty result when checked discovery is malformed |
| `discover_from_manifests_checked` | `root: &Path` | `Result<ManifestDiscovery, String>` | Discover modules while surfacing unreadable or malformed Gradle settings to gate callers |
| `parse_gradle_settings` | `content: &str` | `Result<Vec<GradleSettingsModule>, String>` | Crate-visible shared parser for Groovy/Kotlin includes and project-directory overrides |

### Supported Manifest Types

Seven language ecosystems are supported, each with a dedicated internal parser:

- **Cargo.toml** (Rust) — extracts `[package]` name, `[[bin]]` targets, `[workspace]` members (recursive), `[dependencies]`
- **Package.swift** (Swift) — parses `.target()`, `.executableTarget()` declarations; skips `.testTarget()`; extracts name, path, dependencies params
- **build.gradle.kts / build.gradle** (Kotlin/Java) — detects Android vs standard layout; parses Groovy/Kotlin `include` declarations and `projectDir` overrides from settings.gradle for multi-module projects
- **package.json** (TypeScript/JS) — handles `workspaces` (array or object form) with glob expansion; detects `src/` or `lib/` or `main` field
- **pubspec.yaml** (Dart/Flutter) — extracts `name:` field; defaults source to `lib/`
- **go.mod** (Go) — uses last segment of module path as name; scans for `cmd/`, `internal/`, `pkg/`, `api/` dirs
- **pyproject.toml** (Python) — tries `[project]` then `[tool.poetry]` for name; detects `src/` or package-named dir

General Cargo/Python metadata and Swift declarations use internal string helpers. Security-sensitive
MCP Cargo workspace expansion separately parses bounded manifests as real TOML before consuming
member or target paths.

## Invariants

1. Parsers are tried in a fixed order: Cargo.toml → Package.swift → build.gradle → package.json → pubspec.yaml → go.mod → pyproject.toml
2. Multiple manifest types can coexist — results are merged (first module name wins on conflict)
3. Missing or unreadable ordinary manifest files are skipped; Gradle settings independently
   identify settings-only multi-project workspaces, and checked discovery returns an error when
   present Gradle settings cannot be read or parsed.
4. Cargo workspace members are parsed recursively, with source paths prefixed by the member directory
5. Swift `.testTarget()` declarations are excluded from modules
6. Swift balanced-paren extraction handles nested parentheses correctly
7. Gradle parser distinguishes Android projects (checks `android {` block) from standard Kotlin/Java layouts
8. Gradle multi-module projects support comment-aware Groovy/Kotlin quoting, decoded escapes,
   parenthesized or bare multiline `include` declarations, nested colon names, and literal
   `projectDir` overrides through `file(...)` or `new File(rootDir, ...)`.
9. Every include argument must be a complete quoted literal; dynamic arguments, alternate
   `new File` bases, extra arguments, and trailing assignment expressions fail checked discovery.
10. Included module names and effective `projectDir` values normalize only while they remain
    project-relative; rooted, drive-qualified, UNC, and parent traversal escapes fail checked
    discovery without returning partial modules.
11. `discover_from_manifests_checked` returns an error for malformed or unsupported Gradle forms so
    coverage gates remain inconclusive; it merges the result parsed from that same read instead of
    validating and rereading the path.
12. package.json workspaces support both array form (`["packages/*"]`) and object form (`{ "packages": [...] }`)
13. Go module name uses the last path segment of the module path (e.g. `github.com/user/repo` → `repo`)
14. Python project name resolution tries `[project]` before `[tool.poetry]`
15. General module metadata extraction remains string-based; MCP Cargo workspace preflight uses the
    real TOML parser and rejects malformed workspace shapes without partial discovery.
16. `ManifestDiscovery::default()` returns empty modules and source_dirs

## Behavioral Examples

### Scenario: Rust project with workspace

- **Given** a project root with `Cargo.toml` containing `[workspace] members = ["crates/core", "crates/cli"]`
- **When** `discover_from_manifests(root)` is called
- **Then** returns modules for each workspace member with source paths prefixed (e.g. `crates/core/src`)

### Scenario: Swift package with multiple targets

- **Given** a `Package.swift` declaring `.target(name: "Lib")` and `.executableTarget(name: "CLI")`
- **When** `discover_from_manifests(root)` is called
- **Then** returns both "Lib" and "CLI" as modules with their respective source paths

### Scenario: Node.js monorepo with workspaces

- **Given** `package.json` with `"workspaces": ["packages/*"]` and subdirs `packages/core/` and `packages/web/` each containing a `package.json`
- **When** `discover_from_manifests(root)` is called
- **Then** returns "core" and "web" as modules with source paths like `packages/core/src`

### Scenario: Go project with standard layout

- **Given** `go.mod` with `module github.com/user/myproject` and `cmd/`, `internal/` directories exist
- **When** `discover_from_manifests(root)` is called
- **Then** returns module "myproject" with source dirs `["cmd", "internal"]`

### Scenario: No manifest files present

- **Given** a project root with no recognized manifest files
- **When** `discover_from_manifests(root)` is called
- **Then** returns an empty `ManifestDiscovery` (no modules, no source dirs)

### Scenario: Android Gradle project

- **Given** `build.gradle.kts` containing `android {` and `app/src/main/kotlin/` exists
- **When** `discover_from_manifests(root)` is called
- **Then** includes `app/src/main/kotlin` in source dirs

### Scenario: Gradle module uses a custom project directory

- **Given** settings include `:vendor:member` and map it with `projectDir = file('vendor/custom')`
- **When** `discover_from_manifests(root)` is called
- **Then** the module is named `vendor/member` and its source paths are rooted at `vendor/custom`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Manifest file missing | Parser returns `None`, skipped silently |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) |
| Malformed non-Gradle manifest content | Best-effort extraction; missing fields result in defaults or skipped entries |
| Malformed or dynamic Gradle include, unsupported `projectDir` base/arity/suffix, rooted/drive/UNC/parent-escaping module path, or broken comments/escapes/parentheses | Checked discovery returns `Err`; compatibility discovery returns an empty result and gates stay inconclusive |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) |
| No parsers produce results | Returns default empty `ManifestDiscovery` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| regex | Parse bounded Gradle `projectDir` assignment forms |

### Consumed By

| Module | What is used |
|--------|-------------|
| config | `discover_from_manifests`, `ManifestDiscovery` — for auto-detecting source directories and module structure |
| validator | `ManifestDiscovery` via config's `discover_manifest_modules` — for uncovered-file detection |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-28 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-22 | CHG-0063: Parse standard Groovy/Kotlin Gradle module declarations and effective project directories through one shared parser |
| 2026-07-22 | CHG-0063 follow-up: Add checked, comment/escape-aware Gradle discovery and document real-TOML MCP Cargo workspace preflight |
| 2026-07-22 | CHG-0063 defensive review: Discover settings-only Gradle workspaces and fail closed on malformed settings without requiring a root build script |
| 2026-07-23 | CHG-0063 human review: Reject rooted, drive-qualified, UNC, and parent-escaping Gradle module and `projectDir` paths before CLI discovery can inspect outside the project |
