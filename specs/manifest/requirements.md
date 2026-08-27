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
  whitespace-delimited `new File(rootDir, <literal>)`; dynamic values, alternate bases, extra
  arguments, concatenated `newFile`, indirect/qualified/conditional/block-scoped/compound
  mutators, and trailing expressions reject checked discovery.
- Triple-quoted Groovy/Kotlin documentation and nested comments are inert; unsupported multiline
  values used as directives fail closed instead of creating phantom modules.
- Raw Gradle include identities and raw `project(...)` selectors are rejected when drive-qualified,
  rooted, UNC, or parent-escaping before colon notation is converted to a filesystem path.
- Included module names and effective `projectDir` values must normalize beneath the project root;
  rooted, drive-qualified, UNC, and parent-escaping paths reject checked discovery without partial
  modules. Unicode and Groovy octal escapes are decoded before this confinement decision.
- Every component of a Gradle-derived effective directory is checked no-follow through a retained
  project-root capability before probing or traversal; symlink and Windows reparse-point components
  reject checked discovery without reading the referent.
- Present Gradle build/settings manifests are read through the retained project-root capability,
  including every present lower-precedence filename variant. They must be identity-stable regular
  non-link entries and are bounded to 4 MiB; linked, reparse-backed, non-regular, replaced,
  oversized, unreadable, or invalid-UTF-8 manifests reject checked discovery without partial output.
- Unsupported invoked inclusion APIs such as `includeFlat` and `includeWorkspace` fail closed:
  `includeFlat` resolves against the parent of the root, so its argument is outside the project by
  construction, and `includeWorkspace` is not a form this parser models. `includeBuild` is judged
  by its ARGUMENT instead of its name — one complete literal path confined beneath the project root
  is accepted and then ignored, because a composite build does not alter the root build's own
  `include(...)` list; escapes, interpolated or dynamic expressions, extra arguments, and trailing
  configuration blocks keep failing closed.
  Control-flow rejection applies to governed include/project-directory directives rather than
  unrelated top-level Gradle logic.
- General metadata extraction remains string/regex based; MCP Cargo workspace security discovery parses bounded manifests as real TOML
- `ManifestDiscovery::default()` returns empty collections (safe fallback)
- Checked discovery surfaces malformed Gradle comments, escapes, strings, parentheses, and overrides so coverage gates remain inconclusive
- Caller-retained checked discovery reads every recognized manifest ecosystem and nested workspace
  through one no-follow, non-blocking project capability, with identity continuity, strict UTF-8,
  8 MiB per-file, 64 MiB cumulative, 100,000-entry, and 256-component limits; ambient paths are
  not parser authority.
- Every declared Cargo workspace member and Node workspace pattern consumes a deterministic
  expansion-work entry before traversal; normalized workspace nodes are deduplicated and completed
  results are memoized so repeated declarations cannot cause exponential reparsing.

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

Manifest discovery SHALL identify supported project modules and source roots deterministically
without claiming unsupported workspace expansion.

Acceptance Criteria

- `discover_from_manifests_checked` surfaces malformed Gradle settings to coverage and generation
  gates rather than returning partial discovery, and merges the exact parse from that same read
  instead of validating then rereading a mutable path.
- One parser handles Groovy/Kotlin comments, escapes, literal multiline includes, nested colon
  names, assignment-style `.projectDir = ...`, and method-style `.setProjectDir(...)`.
- Raw include identities and raw `project(...)` selectors are checked before Gradle colon notation
  is mapped to path separators; drive-qualified, rooted, UNC, and parent-escaping spellings reject
  while valid explicitly rooted nested identities remain supported. Unrooted drive-relative
  spellings such as `C:member` reject before colon mapping.
- Assignment and method project-directory values accept exactly `file(<literal>)` or
  whitespace-delimited `new File(rootDir, <literal>)`; concatenated/dynamic lookalikes such as
  `newFile(...)` are unsupported and fail closed.
- Dynamic include arguments, alternate `new File` bases, extra arguments, and trailing assignment
  or method expressions fail checked discovery without returning partial modules.
- Include and project-directory directives are recognized only as top-level executable statements:
  directives inside single-, double-, or triple-quoted strings and nested block/line comments are
  inert, while qualified, aliased, conditional/block-scoped, compound-assignment, and otherwise
  indirect mutations fail closed. Unsupported multiline literals used as directive arguments fail
  rather than disappearing into an empty include.
- Double-quoted `$name` and `${expression}` interpolation fails checked discovery, including when
  Unicode or octal escape decoding reconstructs the dollar. Explicit `\$` and Groovy
  single-quoted dollar literals remain compatible.
- Gradle module identities and `projectDir` literals are confined to project-relative paths:
  rooted, drive-qualified, UNC, and parent-underflow forms fail before source probing, while safe
  literal spellings retain compatibility.
- General module discovery and MCP snapshot preflight use the same effective Gradle module paths.
- Every component of a Gradle-derived effective directory is checked no-follow through a retained
  project-root capability before source probing/traversal; Unix symlink and Windows reparse-point
  components fail checked discovery without reading their referents.
- Present Gradle build/settings manifests are read as regular non-link entries through the retained
  root capability, capped at 4 MiB, and rejected when linked, reparse-backed, non-regular,
  oversized, unreadable, invalid UTF-8, or changed in type during acquisition.
- Every present Gradle build/settings filename is preflighted before precedence selection and its
  native path identity must match the opened handle before and after the bounded read.
- Invoked unsupported inclusion APIs such as `includeFlat` and `includeWorkspace` fail checked
  discovery, while unrelated control flow remains compatible unless it governs an unsupported
  include/project-directory mutation.
- An `includeBuild` naming one literal path beneath the project root parses and contributes no
  module; one naming a path outside the root, or an argument that is not a single complete literal,
  fails checked discovery. This holds wherever the declaration appears — a conditional or
  block-scoped `includeBuild` reaches the same verdict as a top-level one, and one-line and
  multi-line spellings of the same conditional agree.
- A present `settings.gradle[.kts]` is parsed and validated even when no root
  `build.gradle[.kts]` exists.
- MCP Cargo workspace snapshot and confinement discovery parse bounded manifests as real TOML.
- Malformed MCP Cargo TOML/workspace shapes make MCP operations inconclusive; malformed Gradle
  declarations make checked coverage gates inconclusive without partial module results.
- Caller-retained checked discovery acquires Cargo, Swift, Node, Dart, Go, and Python manifests,
  nested Cargo workspace manifests, workspace entries, and source-directory probes through the
  retained project capability. Reads are no-follow, non-blocking, identity-continuous, strict
  UTF-8, and bounded to 8 MiB each/64 MiB cumulatively; directory discovery is sorted and bounded
  to 100,000 entries/256 components. Ambient parser reads are forbidden.
- Every declared Cargo workspace member and Node workspace pattern consumes a deterministic
  expansion-work entry before traversal. Normalized workspace nodes are deduplicated and completed
  results are memoized; budget exhaustion returns an inconclusive checked error without partial
  modules or repeated subtree parsing.
- Nested manifest/workspace directories remain reachable through the retained project root after
  enumeration and before/after reads; detached-parent replacement fails without mixing
  generations.


### REQ-manifest-002

A Gradle project's module identity SHALL come from its project name.

Acceptance Criteria
- A single-project build is named from a literal `rootProject.name`.
- When `rootProject.name` is unset the project directory name is used, which is Gradle's own default rather than a spec-sync convention.
- A multi-project build continues to use its `include` names.
- No module name is derived from a source path segment, so neither the first nor the last segment of a package hierarchy can become a module.

### REQ-manifest-018

A manifest module SHALL carry the source paths it declares, so that a consumer can judge the module against its own files rather than against its name alone.

Acceptance Criteria
- Every discovered manifest module exposes the source paths attributed to it by the manifest it came from.
- A module whose manifest declares no source paths exposes an empty set rather than being omitted, so a consumer can tell "declares nothing" from "was not discovered".
- Cargo, Swift and Gradle discovery all populate the field, including the Gradle single-project fallback that derives its name from the root directory.

