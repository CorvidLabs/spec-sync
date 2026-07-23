---
spec: manifest.spec.md
---

## Key Decisions

- **Process-free parsing with structural Cargo security checks**: General metadata extraction remains string/regex based. MCP snapshot and confinement discovery parse bounded Cargo manifests with the in-process `toml` crate; no manifest discovery launches a process.
- **Fixed parse order, merged results**: Parsers run in a fixed sequence; results are merged with first-module-name-wins on conflict. This keeps behavior deterministic across runs.
- **Checked Gradle failure**: Missing ordinary manifests remain a normal skip. Present unreadable or malformed Gradle settings return `Err` from `discover_from_manifests_checked`; the checked path merges the exact single-read parse result, while the compatibility wrapper collapses failure to an empty result.
- **Settings are independently authoritative**: A root `settings.gradle[.kts]` is sufficient to
  discover a Gradle multi-project workspace. Its modules and parse failures are never hidden by a
  missing root `build.gradle[.kts]`.
- **Workspace/monorepo support**: Cargo workspaces, package.json workspaces, and Gradle multi-module projects are handled. Cargo security preflight uses real TOML; Gradle settings use one comment/escape-aware parser for Groovy/Kotlin include forms and effective `projectDir` mappings so MCP preflight and general discovery agree.
- **Fail-closed Gradle subset**: Include arguments must be complete string literals. Supported
  `projectDir` expressions are exactly `file(<literal>)` and `new File(rootDir, <literal>)` with no
  trailing expression; dynamic arguments, alternate bases, extra arguments, and any rooted,
  drive-qualified, UNC, or parent-escaping normalized module path reject the parse.
- **Swift test target exclusion**: `.testTarget()` entries are explicitly skipped to avoid polluting the module list with test infrastructure.
- **Python priority**: `[project]` section is checked before `[tool.poetry]` in pyproject.toml, reflecting the ecosystem's migration toward PEP 621.

## Files to Read First

- `src/manifest.rs` — Single-file module with a parser function per language ecosystem.

## Current Status

Implemented for all 7 manifest formats. Gate callers use `discover_from_manifests_checked()` so malformed Gradle discovery is inconclusive instead of a compatibility fallback. Gradle module paths are normalized from colon notation and custom project directories before source probing, including settings-only workspaces, and normalization fails before discovery when a path would escape the project root; MCP Cargo workspace paths come from validated TOML values.

## Notes

- Balanced parenthesis extraction is used for Swift's `Package.swift` — it's a mini expression parser for the `.target(name: ..., path: ...)` syntax.
- Go module detection uses the last path segment of the module name (e.g., `github.com/user/repo` → `repo`) and probes for standard directory conventions (cmd, internal, pkg, api).
