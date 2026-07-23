---
spec: manifest.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/manifest.rs` | cargo test manifest:: | Core ecosystem parsers plus shared Gradle settings, comments/escapes, effective project directories, and checked malformed-discovery regressions |

## Coverage Gaps

- Integration gap: add a fixture for "Rust project with workspace" before changing user-visible CLI output, generated files, or error handling in manifest.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Rust project with workspace | a project root with `Cargo.toml` containing `[workspace] members = ["crates/core", "crates/cli"]` | `discover_from_manifests(root)` is called | returns modules for each workspace member with source paths prefixed (e.g. `crates/core/src`) |
| Swift package with multiple targets | a `Package.swift` declaring `.target(name: "Lib")` and `.executableTarget(name: "CLI")` | `discover_from_manifests(root)` is called | returns both "Lib" and "CLI" as modules with their respective source paths |
| Node.js monorepo with workspaces | `package.json` with `"workspaces": ["packages/*"]` and subdirs `packages/core/` and `packages/web/` each containing a `package.json` | `discover_from_manifests(root)` is called | returns "core" and "web" as modules with source paths like `packages/core/src` |
| Go project with standard layout | `go.mod` with `module github.com/user/myproject` and `cmd/`, `internal/` directories exist | `discover_from_manifests(root)` is called | returns module "myproject" with source dirs `["cmd", "internal"]` |
| No manifest files present | a project root with no recognized manifest files | `discover_from_manifests(root)` is called | returns an empty `ManifestDiscovery` (no modules, no source dirs) |
| Android Gradle project | `build.gradle.kts` containing `android {` and `app/src/main/kotlin/` exists | `discover_from_manifests(root)` is called | includes `app/src/main/kotlin` in source dirs |
| Standard Gradle settings variants | comments, escaped Groovy/Kotlin quotes, literal multiline includes, nested names, and the two supported literal `projectDir` forms | `parse_gradle_settings` and checked discovery are called | each unique module is returned once with its effective normalized source directory |
| Settings-only Gradle workspace | `settings.gradle[.kts]` exists without a root build script | checked discovery is called | included modules are discovered; malformed settings return `Err` instead of empty coverage |
| Cargo workspace security discovery | multiline TOML workspace members with comments or a commented fake `[workspace]` header | MCP snapshot/preflight discovery is called | real TOML members are included and malformed TOML is inconclusive without partial paths |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Manifest file missing | Parser returns `None`, skipped silently | Keep or add a focused assertion before changing this behavior |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) | Keep or add a focused assertion before changing this behavior |
| Malformed non-Gradle manifest content | Best-effort extraction; missing fields result in defaults or skipped entries | Keep or add a focused assertion before changing this behavior |
| Malformed Gradle comments, escapes, strings, parentheses, or override | Checked discovery returns `Err` and coverage gates remain inconclusive | Exercise each malformed class without partial module results; checked discovery merges the same parsed read rather than rereading |
| Missing root Gradle build script with present settings | Settings remain authoritative | Exercise valid and malformed settings-only workspaces |
| Dynamic Gradle include or unsupported `projectDir` base/arity/suffix | Checked discovery returns `Err` without partial modules | Exercise mixed literal/dynamic includes, `rootDir.parentFile`, extra arguments, and trailing expressions |
| Rooted, drive-qualified, UNC, or parent-escaping Gradle module/`projectDir` path | Checked discovery returns `Err` without inspecting outside the project or returning partial modules | Exercise `file("../outside")`, `include(":..:x")`, nested traversal, drive paths, and UNC paths |
| Malformed MCP Cargo TOML or workspace shape | Snapshot/confinement discovery is inconclusive | Exercise syntax errors, non-table workspace, and non-string members |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) | Keep or add a focused assertion before changing this behavior |
| No parsers produce results | Returns default empty `ManifestDiscovery` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/manifest.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
