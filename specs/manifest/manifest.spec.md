---
module: manifest
version: 9
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

### Exported Constants

| Constant | Type | Description |
|----------|------|-------------|
| `MAX_GRADLE_MANIFEST_BYTES` | `u64` | Crate-visible 4 MiB ceiling shared by retained Gradle manifest readers |

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
| `parse_gradle_settings` | `content: &str` | `Result<Vec<GradleSettingsModule>, String>` | Crate-visible shared parser for Groovy/Kotlin includes plus assignment-style and method-style literal project-directory overrides |

### Supported Manifest Types

Seven language ecosystems are supported, each with a dedicated internal parser:

- **Cargo.toml** (Rust) — extracts `[package]` name, `[[bin]]` targets, `[workspace]` members (recursive), `[dependencies]`
- **Package.swift** (Swift) — parses `.target()`, `.executableTarget()` declarations; skips `.testTarget()`; extracts name, path, dependencies params
- **build.gradle.kts / build.gradle** (Kotlin/Java) — detects Android vs standard layout; parses Groovy/Kotlin `include` declarations plus assignment-style and method-style literal project-directory overrides from settings.gradle for multi-module projects
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
3. Missing ordinary manifest files are skipped. Present Gradle build/settings manifests are opened
   through the retained project-root capability, must be regular non-link entries, are bounded to
   4 MiB, and are parsed from the exact retained bytes. Linked, reparse-backed, non-regular,
   oversized, unreadable, or invalid-UTF-8 Gradle manifests fail checked discovery.
4. Cargo workspace members are parsed recursively, with source paths prefixed by the member directory
5. Swift `.testTarget()` declarations are excluded from modules
6. Swift balanced-paren extraction handles nested parentheses correctly
7. Gradle parser distinguishes Android projects (checks `android {` block) from standard Kotlin/Java layouts
8. Gradle multi-module projects support comment-aware Groovy/Kotlin quoting, decoded escapes,
   parenthesized or bare multiline `include` declarations, nested colon names, assignment-style
   `.projectDir = ...`, and method-style `.setProjectDir(...)`. Triple-quoted Groovy/Kotlin
   documentation and nested block comments are inert.
9. Supported assignment and method arguments are exactly `file(<literal>)` and
   `new File(rootDir, <literal>)`. Every include argument and project-directory argument must be a
   complete literal expression; `new File` requires a real token boundary. Unescaped interpolation
   in double-quoted strings, dynamic arguments, alternate bases, extra arguments, aliased,
   qualified, conditional, block-scoped, compound, or otherwise unsupported mutations, and
   trailing expressions fail checked discovery. Escaped literal dollars remain data, while Unicode
   and Groovy octal escapes are decoded before confinement checks.
10. Raw included module identities and raw `project(...)` selectors are checked for rooted,
    drive-qualified (including drive-relative `C:member`), UNC, and parent-escaping forms before
    Gradle colon separators are mapped to path separators. Valid rooted nested identities such as
    `:service:api` and `:C:member` remain supported.
11. Included module names and effective project-directory values normalize only while they remain
    project-relative; rooted, drive-qualified, UNC, and parent traversal escapes fail checked
    discovery without returning partial modules.
12. Every filesystem component of a Gradle-derived effective directory is resolved through the
    retained project-root capability with no-follow semantics before source probing or traversal.
    A symlink or Windows reparse point at any component makes checked discovery inconclusive; its
    referent is never used as a source root.
13. `discover_from_manifests_checked` returns an error for malformed or unsupported Gradle forms so
    coverage gates remain inconclusive; it merges the result parsed from that same read instead of
    validating and rereading the path.
14. package.json workspaces support both array form (`["packages/*"]`) and object form (`{ "packages": [...] }`)
15. Go module name uses the last path segment of the module path (e.g. `github.com/user/repo` → `repo`)
16. Python project name resolution tries `[project]` before `[tool.poetry]`
17. General module metadata extraction remains string-based; MCP Cargo workspace preflight uses the
    real TOML parser and rejects malformed workspace shapes without partial discovery.
18. `ManifestDiscovery::default()` returns empty modules and source_dirs

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

### Scenario: Gradle uses the official project-directory mutator

- **Given** settings include `:vendor:member` and call
  `project(":vendor:member").setProjectDir(file("vendor/custom"))`
- **When** `discover_from_manifests_checked(root)` is called
- **Then** the module is named `vendor/member` and its source paths are rooted at `vendor/custom`

### Scenario: Gradle-derived directory contains a link

- **Given** a Gradle module resolves through a symlink or Windows reparse point beneath the project
  root
- **When** checked manifest discovery is called
- **Then** discovery returns an error before source probing or traversal and does not inspect the
  link target

### Scenario: Gradle manifest is a link

- **Given** `build.gradle[.kts]` or `settings.gradle[.kts]` is a symlink or Windows reparse point
- **When** checked manifest discovery is called
- **Then** discovery returns an inconclusive error without reading or disclosing referent bytes

### Scenario: Gradle path uses interpolation or encoded traversal

- **Given** a Gradle include or project-directory expression uses `$name`, `${expression}`,
  `\u002e`, or Groovy octal escapes
- **When** checked manifest discovery is called
- **Then** interpolation is rejected as dynamic and decoded traversal is rejected before partial
  discovery, source probing, or generation

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Manifest file missing | Parser returns `None`, skipped silently |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) |
| Malformed non-Gradle manifest content | Best-effort extraction; missing fields result in defaults or skipped entries |
| Linked, reparse-backed, non-regular, oversized, unreadable, or invalid-UTF-8 Gradle build/settings manifest | Checked discovery returns `Err` without reading a link referent or returning partial discovery; compatibility discovery returns an empty result |
| Malformed or dynamic Gradle include, unescaped double-quoted interpolation, unsupported assignment/method project-directory form, rooted/drive/UNC/parent-escaping raw module identity or decoded effective path, or broken comments/escapes/parentheses | Checked discovery returns `Err`; compatibility discovery returns an empty result and gates stay inconclusive |
| Gradle-derived directory contains a symlink or Windows reparse-point component | Checked discovery returns `Err` before source probing/traversal; compatibility discovery returns an empty result and gates stay inconclusive |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) |
| No parsers produce results | Returns default empty `ManifestDiscovery` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| cap-std | Retained project-root capability for bounded no-follow Gradle manifest reads and component inspection of derived source roots |
| regex | Locate bounded Gradle assignment-style and method-style project-directory forms |

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
| 2026-07-23 | v7 / CHG-0063 independent review: Validate raw drive-qualified module identities before colon mapping, confine literal `setProjectDir` forms, and reject symlink/reparse components through the retained root capability |
| 2026-07-23 | v8 / CHG-0063 adversarial rereview: Acquire Gradle build/settings manifests as bounded regular non-link files, reject unescaped interpolation, and decode Unicode/octal path escapes before confinement |
| 2026-07-23 | v9 / CHG-0063 final security rereview: Reject indirect/conditional Gradle mutations, mask multiline literals and nested comments, require the `new File` token boundary, and preserve only explicitly rooted Gradle names that resemble drive-relative paths |
