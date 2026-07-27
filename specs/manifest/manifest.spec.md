---
module: manifest
version: 16
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

## Invariants

1. Gradle settings parsing is comment- and escape-aware and supports literal Groovy/Kotlin multiline
   include declarations.
2. Raw include identities and project selectors reject drive-qualified, rooted, UNC, and
   parent-escaping forms before colon-to-path conversion.
3. Nested colon names and the supported literal assignment/method project-directory forms resolve
   to one deterministic effective project-relative directory per module.
4. Dynamic, qualified, aliased, conditional/block-scoped, or otherwise indirect includes and
   `projectDir`/`setProjectDir` mutations, unsupported bases/arity/suffixes, compound assignments,
   and unsupported multiline directive arguments fail without partial discovery.
5. Double-quoted Gradle interpolation is rejected after escape decoding; explicit escaped-dollar
   and Groovy single-quoted literal-dollar forms remain deterministic literals.
6. Checked discovery reports malformed Gradle input; compatibility discovery may return an empty
   result, but gate callers remain inconclusive.
7. Checked Gradle discovery merges the exact single-read parse result.
8. MCP Cargo workspace discovery trusts only structurally parsed TOML members and target paths.
9. Settings-only Gradle workspaces discover included modules, while malformed settings remain an
   inconclusive checked-discovery error.
10. Gradle module and effective project-directory paths cannot traverse above the project root or
   select rooted, drive-qualified, or UNC locations; rejection occurs before partial discovery or
   filesystem probing.
11. Every Gradle-derived directory component is inspected no-follow through the retained root
    capability; symlink and reparse-point components reject before source probing or traversal.
12. Present Gradle build/settings manifests are bounded regular non-link retained-capability reads;
    malformed endpoints or bytes reject before partial discovery.
13. Every present filename variant is preflighted before precedence and remains identity-stable
    through open/read; unsafe shadowed variants cannot evade checked discovery.
14. Unsupported invoked inclusion APIs and governed indirect/conditional mutations fail closed,
    while unrelated Gradle control flow and identifier/documentation uses remain compatible.
15. Every recognized checked manifest ecosystem and nested workspace probe uses the caller's
    retained project capability with deterministic byte, entry, depth, UTF-8, link, special-file,
    and identity enforcement; ambient paths are only a final replacement diagnostic.
16. Cargo/Node workspace expansion charges declarations independently of unique retained bytes,
    deduplicates normalized nodes, and reuses completed results.
17. Retained nested manifest/workspace parents are reverified through the project root around
    enumeration and reads.
18. Node workspace enumeration records child identities, opens children sequentially through the
    retained workspace base, and consumes child manifests/source probes only from identity-matching
    capabilities. Each verified base listing is released before the next distinct base, so
    swap/read/restore cannot mix generations and neither sibling nor base breadth exhausts handles.

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

### Scenario: Duplicate workspace declarations

- **Given** Cargo manifests or Node workspace arrays repeat the same normalized child declarations
  at multiple levels
- **When** checked manifest discovery expands the workspace graph
- **Then** each declaration is charged, each normalized child is completed once, and exhausted work
  bounds return an inconclusive error instead of repeatedly parsing the same subtree

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

### Scenario: A shadowed Gradle variant is unsafe

- **Given** `settings.gradle.kts` is selected by precedence while a present `settings.gradle` is a
  directory, link/reparse point, special file, replacement, oversized file, or invalid UTF-8
- **When** checked manifest discovery is called
- **Then** discovery fails before parsing or source probing; the lower-precedence variant cannot
  evade security preflight

### Scenario: Unrelated Gradle control flow

- **Given** a settings file contains unrelated top-level `if`, `for`, or closure logic plus a
  supported top-level literal `include`
- **When** checked manifest discovery is called
- **Then** the supported include is parsed; only indirect, conditional, or otherwise unsupported
  include/project-directory mutations are rejected

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
| Non-Gradle manifest is unsafe, replaced, invalid UTF-8, over 8 MiB, or retained discovery exceeds 64 MiB/100,000 entries/256 components | Caller-retained checked discovery returns `Err` without ambient fallback or partial discovery; compatibility discovery returns an empty result |
| Cargo/Node workspace declarations exceed the expansion budget or repeat a completed normalized node | Checked discovery returns `Err` on budget exhaustion; otherwise it reuses the completed result without reparsing the subtree |
| Nested manifest/workspace parent is detached or replaced during enumeration/read | Caller-retained checked discovery returns `Err` after project-root reachability verification; detached and replacement generations are not mixed |
| Enumerated Node workspace is swapped during a child read, or sibling/base breadth exceeds the process descriptor limit | Child bytes/probes come from an identity-matching retained capability opened sequentially; completed base listings are released, so replacement generations are not mixed and handles remain bounded |
| Malformed non-Gradle manifest content | Best-effort extraction; missing fields result in defaults or skipped entries |
| Linked, reparse-backed, non-regular, replaced, oversized, unreadable, or invalid-UTF-8 Gradle build/settings manifest, including a shadowed filename variant | Checked discovery returns `Err` without reading a link referent or returning partial discovery; compatibility discovery returns an empty result |
| Malformed or dynamic Gradle include, invoked unsupported inclusion API, unescaped double-quoted interpolation, unsupported assignment/method project-directory form, rooted/drive/UNC/parent-escaping raw module identity or decoded effective path, or broken comments/escapes/parentheses | Checked discovery returns `Err`; compatibility discovery returns an empty result and gates stay inconclusive |
| Gradle-derived directory contains a symlink or Windows reparse-point component | Checked discovery returns `Err` before source probing/traversal; compatibility discovery returns an empty result and gates stay inconclusive |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) |
| No parsers produce results | Returns default empty `ManifestDiscovery` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| cap-std | Retained project-root capability for bounded no-follow reads and directory inspection across every recognized checked manifest ecosystem |
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
| 2026-07-23 | v10 / CHG-0063 post-review hardening: Preflight every present Gradle filename including shadowed variants, bind manifest identity across open/read, scope control-flow rejection to governed directives, and reject invoked unsupported inclusion APIs |
| 2026-07-23 | v11 / CHG-0063 acceptance remediation: Acquire all recognized checked manifests, nested workspaces, and manifest probes through one bounded retained project capability without ambient parser fallback |
| 2026-07-23 | v12 / CHG-0063 exact-head review remediation: Bound declared Cargo/Node workspace expansion, memoize completed normalized nodes, and verify nested manifest/workspace reachability so duplicate declarations or detached parents cannot produce mixed discovery |
| 2026-07-24 | v13 / CHG-0063 independent rereview remediation: Parse retained Cargo and Node workspace declarations structurally, charge malformed entries before rejection, and bind nested workspace directory listings through child consumption |
| 2026-07-24 | v14 / CHG-0063 exact-head rereview remediation: Consume Node child manifests and probes through identity-bound enumerated capabilities while bounding live handles independently of sibling count |
| 2026-07-24 | v15 / CHG-0063 descriptor-breadth remediation: Release each verified Node workspace-base listing so live handles remain bounded across distinct base patterns as well as sibling directories |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
