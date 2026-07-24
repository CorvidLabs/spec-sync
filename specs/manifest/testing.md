---
spec: manifest.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/manifest.rs` | cargo test manifest:: | Core ecosystem parsers plus shared Gradle settings, raw drive-prefix checks, assignment/method project-directory parsing, bounded no-follow manifest reads, no-follow effective directories, and checked malformed-discovery regressions |

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
| Official Gradle project-directory method | literal `setProjectDir(file("..."))` and `setProjectDir(new File(rootDir, "..."))` calls | `parse_gradle_settings` and checked discovery are called | both literal forms map the selected module to one confined effective directory |
| Gradle-derived link component | a module or effective project directory contains a Unix symlink or Windows reparse point | checked discovery is called | discovery is inconclusive before source probing/traversal and outside sentinel bytes remain unchanged |
| Interpolated or encoded Gradle path | double-quoted `$name`/`${expression}`, `\u002e`, or Groovy octal escapes | checked discovery is called | interpolation is rejected; decoded traversal is confined; escaped/single-quoted literal dollars remain compatible |
| Linked Gradle manifest | `build.gradle[.kts]` or `settings.gradle[.kts]` links outside the project | each checked CLI coverage gate runs | each gate fails inconclusively with valid structured output, no referent disclosure, no project mutation, and unchanged outside bytes |
| Settings-only Gradle workspace | `settings.gradle[.kts]` exists without a root build script | checked discovery is called | included modules are discovered; malformed settings return `Err` instead of empty coverage |
| Cargo workspace security discovery | multiline TOML workspace members with comments or a commented fake `[workspace]` header | MCP snapshot/preflight discovery is called | real TOML members are included and malformed TOML is inconclusive without partial paths |
| Ambient project pathname replaced after capability retention | original Cargo manifest remains reachable only through the retained directory while the ambient name points at attacker bytes | retained checked discovery runs | only the retained manifest identity and bytes are parsed; replacement module bytes are absent |

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
| Raw drive-qualified Gradle include or project selector | Rejected before `:` is mapped to `/`, while valid nested colon identities remain accepted | Exercise `include("C:/outside")`, `include(":C:/outside")`, drive-qualified `project(...)`, and `:service:api` |
| Literal or dynamic `setProjectDir` call | The two supported literal forms are confined; dynamic/unsupported forms fail without partial modules | Exercise `file(...)`, `new File(rootDir, ...)`, variables, alternate bases, extra arguments, and trailing expressions |
| Double-quoted interpolation or encoded traversal | Dynamic interpolation rejects; safe literal dollars survive; decoded Unicode/octal `..` cannot bypass confinement | Exercise include, assignment, and setter forms through parser plus CLI/MCP gates |
| Symlink or Windows reparse point in a Gradle-derived directory | Checked discovery fails before source probing/traversal and does not disclose referent content | Exercise every-path-component no-follow checks on Unix and hosted Windows; preserve outside sentinel bytes |
| Linked/reparse-backed or oversized Gradle build/settings manifest | Checked discovery fails before parsing or partial output and never reads a link referent | Exercise both build and settings names through check/coverage/generate/report/score, plus hosted-Windows reparse runtime and the 4 MiB bound |
| Malformed MCP Cargo TOML or workspace shape | Snapshot/confinement discovery is inconclusive | Exercise syntax errors, non-table workspace, and non-string members |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) | Keep or add a focused assertion before changing this behavior |
| No parsers produce results | Returns default empty `ManifestDiscovery` | Keep or add a focused assertion before changing this behavior |
| Retained non-Gradle manifest or workspace input is replaced, linked, special, oversized, invalid UTF-8, or over budget | Checked discovery fails or consumes only the identity-continuous retained bytes; no ambient fallback | Keep `retained_non_gradle_manifest_access_ignores_an_ambient_root_replacement` plus focused bounds/identity tests |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/manifest.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
- Treat all CHG-0063 amendment regressions, hosted-Windows runtime, independent rereviews,
  repository/CI, trust, and Attest evidence as pending until rerun on the final tree.
