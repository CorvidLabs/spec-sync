---
module: manifest
version: 1
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

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `discover_from_manifests` | `root: &Path` | `ManifestDiscovery` | Parse all supported manifest files in the project root and return discovered modules and source directories |

### Supported Manifest Types

Seven language ecosystems are supported, each with a dedicated internal parser:

- **Cargo.toml** (Rust) — extracts `[package]` name, `[[bin]]` targets, `[workspace]` members (recursive), `[dependencies]`
- **Package.swift** (Swift) — parses `.target()`, `.executableTarget()` declarations; skips `.testTarget()`; extracts name, path, dependencies params
- **build.gradle.kts / build.gradle** (Kotlin/Java) — detects Android vs standard layout; parses `include()` from settings.gradle for multi-module projects
- **package.json** (TypeScript/JS) — handles `workspaces` (array or object form) with glob expansion; detects `src/` or `lib/` or `main` field
- **pubspec.yaml** (Dart/Flutter) — extracts `name:` field; defaults source to `lib/`
- **go.mod** (Go) — uses last segment of module path as name; scans for `cmd/`, `internal/`, `pkg/`, `api/` dirs
- **pyproject.toml** (Python) — tries `[project]` then `[tool.poetry]` for name; detects `src/` or package-named dir

Internal TOML/Swift helpers handle section extraction, balanced parentheses, and string parameter parsing without external TOML/Swift parser dependencies.

## Invariants

1. Parsers are tried in a fixed order: Cargo.toml → Package.swift → build.gradle → package.json → pubspec.yaml → go.mod → pyproject.toml
2. Multiple manifest types can coexist — results are merged (first module name wins on conflict)
3. Missing or unreadable manifest files return `None` (never error)
4. Cargo workspace members are parsed recursively, with source paths prefixed by the member directory
5. Swift `.testTarget()` declarations are excluded from modules
6. Swift balanced-paren extraction handles nested parentheses correctly
7. Gradle parser distinguishes Android projects (checks `android {` block) from standard Kotlin/Java layouts
8. Gradle multi-module projects are detected via `include()` in `settings.gradle.kts` / `settings.gradle`
9. package.json workspaces support both array form (`["packages/*"]`) and object form (`{ "packages": [...] }`)
10. Go module name uses the last path segment of the module path (e.g. `github.com/user/repo` → `repo`)
11. Python project name resolution tries `[project]` before `[tool.poetry]`
12. TOML parsing is regex/string-based (no TOML library dependency) — handles common patterns but not full TOML spec
13. `ManifestDiscovery::default()` returns empty modules and source_dirs

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

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Manifest file missing | Parser returns `None`, skipped silently |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) |
| Malformed manifest content | Best-effort extraction; missing fields result in defaults or skipped entries |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) |
| No parsers produce results | Returns default empty `ManifestDiscovery` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| *(none)* | Self-contained — uses only `std::collections`, `std::fs`, `std::path`, and `serde_json` |

### Consumed By

| Module | What is used |
|--------|-------------|
| config | `discover_from_manifests`, `ManifestDiscovery` — for auto-detecting source directories and module structure |
| validator | `ManifestDiscovery` via config's `discover_manifest_modules` — for uncovered-file detection |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-28 | Initial spec |
