---
spec: manifest.spec.md
---

## User Stories

- As a Rust developer, I want spec-sync to parse my Cargo.toml (including workspace members) so that all crates are discovered as modules automatically
- As a TypeScript developer, I want package.json workspaces (both array and object forms) to be expanded so that monorepo packages are detected
- As a Swift developer, I want Package.swift targets parsed so that each target becomes a discoverable module
- As a Go developer, I want go.mod parsed so that my module name and conventional directories (cmd/, internal/, pkg/) are discovered
- As a Python developer, I want pyproject.toml parsed so that package source directories are found automatically
- As a Kotlin/Java developer, I want build.gradle.kts and settings.gradle parsed so that multi-module Gradle projects are detected
- As a Dart/Flutter developer, I want pubspec.yaml parsed so that the lib/ source directory is detected

## Acceptance Criteria

- [ ] Seven manifest types supported: Cargo.toml, Package.swift, build.gradle.kts, package.json, pubspec.yaml, go.mod, pyproject.toml
- [ ] Parsers are tried in fixed order with results merged (first wins on name conflicts)
- [ ] Cargo workspace members are parsed recursively with source paths prefixed by member directory
- [ ] package.json workspaces support both array and object forms with glob expansion
- [ ] Go module name uses the last path segment of the module path
- [ ] Python tries `[project]` before `[tool.poetry]` in pyproject.toml
- [ ] Gradle multi-module detection via `include()` in settings.gradle
- [ ] All TOML/YAML parsing is zero-dependency (regex and string-based)
- [ ] `ManifestDiscovery::default()` returns empty collections (safe fallback)
- [ ] Unparseable manifests are silently skipped (no errors, try next format)

## Constraints

- No external TOML, YAML, or Swift package description parser dependencies
- Parsing must be fast — only string/regex operations, no process spawning
- Must handle malformed manifests gracefully without panicking

## Out of Scope

- Parsing lock files (Cargo.lock, package-lock.json, etc.)
- Resolving transitive dependencies
- Supporting manifest formats for C/C++ (CMake, Makefile), .NET (csproj), or other build systems
- Downloading or fetching remote workspace members
