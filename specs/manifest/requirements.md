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

- Seven manifest types supported: Cargo.toml, Package.swift, build.gradle.kts, package.json, pubspec.yaml, go.mod, pyproject.toml
- Parsers are tried in fixed order with results merged (first wins on name conflicts)
- Cargo workspace members are parsed recursively with source paths prefixed by member directory
- package.json workspaces support both array and object forms with glob expansion
- Go module name uses the last path segment of the module path
- Python tries `[project]` before `[tool.poetry]` in pyproject.toml
- Gradle multi-module detection accepts comment-aware Groovy/Kotlin single- and double-quoted,
  escape-decoded, parenthesized or bare multiline `include` declarations, nested colon paths, and
  assignment-style `projectDir` and method-style `setProjectDir` overrides.
- Every Gradle `include` argument is a complete quoted literal; dynamic or mixed expressions reject
  checked discovery instead of returning a partial module set. Unescaped `$name`/`${expression}`
  interpolation in double-quoted strings is dynamic; escaped or single-quoted literal dollars
  remain data.
- Supported assignment and method values are exactly `file(<literal>)` or
  `new File(rootDir, <literal>)`; dynamic values, alternate bases, extra arguments, unsupported
  mutators, and trailing expressions reject checked discovery.
- Raw Gradle include identities and raw `project(...)` selectors are rejected when drive-qualified,
  rooted, UNC, or parent-escaping before colon notation is converted to a filesystem path.
- Included module names and effective `projectDir` values must normalize beneath the project root;
  rooted, drive-qualified, UNC, and parent-escaping paths reject checked discovery without partial
  modules. Unicode and Groovy octal escapes are decoded before this confinement decision.
- Every component of a Gradle-derived effective directory is checked no-follow through a retained
  project-root capability before probing or traversal; symlink and Windows reparse-point components
  reject checked discovery without reading the referent.
- Present Gradle build/settings manifests are read through the retained project-root capability,
  must be regular non-link entries, and are bounded to 4 MiB; linked, reparse-backed, non-regular,
  oversized, unreadable, or invalid-UTF-8 manifests reject checked discovery without partial output.
- General metadata extraction remains string/regex based; MCP Cargo workspace security discovery parses bounded manifests as real TOML
- `ManifestDiscovery::default()` returns empty collections (safe fallback)
- Checked discovery surfaces malformed Gradle comments, escapes, strings, parentheses, and overrides so coverage gates remain inconclusive

## Constraints

- General manifest metadata avoids external YAML or Swift package parsers; MCP security preflight uses the `toml` crate for Cargo workspace structure
- Parsing must be local and process-free; general discovery uses string/regex operations and MCP Cargo preflight uses in-process TOML
- Must handle malformed manifests without panicking or returning partial security-gate discovery

## Out of Scope

- Parsing lock files (Cargo.lock, package-lock.json, etc.)
- Resolving transitive dependencies
- Supporting manifest formats for C/C++ (CMake, Makefile), .NET (csproj), or other build systems
- Downloading or fetching remote workspace members

### REQ-manifest-001

Manifest discovery SHALL identify supported project modules and source roots deterministically without claiming unsupported workspace expansion.

Acceptance Criteria
- Seven manifest types supported: Cargo.toml, Package.swift, build.gradle.kts, package.json, pubspec.yaml, go.mod, pyproject.toml
- Parsers are tried in fixed order with results merged (first wins on name conflicts)
- Cargo workspace members are parsed recursively with source paths prefixed by member directory
- package.json workspaces support both array and object forms with glob expansion
- Go module name uses the last path segment of the module path
- Python tries `[project]` before `[tool.poetry]` in pyproject.toml
- Gradle multi-module detection accepts comment-aware Groovy/Kotlin single- and double-quoted,
  escape-decoded, parenthesized or bare multiline `include` declarations, nested colon paths, and
  assignment-style `projectDir` and method-style `setProjectDir` overrides.
- Every Gradle `include` argument is a complete quoted literal; dynamic or mixed expressions reject
  checked discovery instead of returning a partial module set. Unescaped `$name`/`${expression}`
  interpolation in double-quoted strings is dynamic; escaped or single-quoted literal dollars
  remain data.
- Supported assignment and method values are exactly `file(<literal>)` or
  `new File(rootDir, <literal>)`; dynamic values, alternate bases, extra arguments, unsupported
  mutators, and trailing expressions reject checked discovery.
- Raw Gradle include identities and raw `project(...)` selectors are rejected when drive-qualified,
  rooted, UNC, or parent-escaping before colon notation is converted to a filesystem path.
- Included module names and effective `projectDir` values must normalize beneath the project root;
  rooted, drive-qualified, UNC, and parent-escaping paths reject checked discovery without partial
  modules. Unicode and Groovy octal escapes are decoded before this confinement decision.
- Every component of a Gradle-derived effective directory is checked no-follow through a retained
  project-root capability before probing or traversal; symlink and Windows reparse-point components
  reject checked discovery without reading the referent.
- Present Gradle build/settings manifests are read through the retained project-root capability,
  must be regular non-link entries, and are bounded to 4 MiB; linked, reparse-backed, non-regular,
  oversized, unreadable, or invalid-UTF-8 manifests reject checked discovery without partial output.
- General metadata extraction remains string/regex based; MCP Cargo workspace security discovery parses bounded manifests as real TOML
- `ManifestDiscovery::default()` returns empty collections (safe fallback)
- Checked discovery surfaces malformed Gradle comments, escapes, strings, parentheses, and overrides so coverage gates remain inconclusive
