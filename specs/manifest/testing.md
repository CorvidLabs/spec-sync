---
spec: manifest.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/manifest.rs` | cargo test manifest:: | `test_parse_cargo_toml_basic`, `test_parse_package_swift_basic`, `test_parse_package_json_workspaces`, `test_parse_go_mod`, `test_extract_balanced_parens` |

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

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Manifest file missing | Parser returns `None`, skipped silently | Keep or add a focused assertion before changing this behavior |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) | Keep or add a focused assertion before changing this behavior |
| Malformed manifest content | Best-effort extraction; missing fields result in defaults or skipped entries | Keep or add a focused assertion before changing this behavior |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) | Keep or add a focused assertion before changing this behavior |
| No parsers produce results | Returns default empty `ManifestDiscovery` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/manifest.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
