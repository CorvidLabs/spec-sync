---
spec: manifest.spec.md
---

## Key Decisions

- **Zero-dependency parsing**: All 7 manifest formats (Cargo.toml, Package.swift, build.gradle, package.json, pubspec.yaml, go.mod, pyproject.toml) are parsed with regex and string operations. No TOML, YAML, or Swift parser crates.
- **Fixed parse order, merged results**: Parsers run in a fixed sequence; results are merged with first-module-name-wins on conflict. This keeps behavior deterministic across runs.
- **Silent failure**: Missing or unreadable manifest files return `None` silently — a project might have some manifests but not others, and that's normal.
- **Workspace/monorepo support**: Cargo workspaces, package.json workspaces, and Gradle multi-module projects are all handled, with recursive member discovery.
- **Swift test target exclusion**: `.testTarget()` entries are explicitly skipped to avoid polluting the module list with test infrastructure.
- **Python priority**: `[project]` section is checked before `[tool.poetry]` in pyproject.toml, reflecting the ecosystem's migration toward PEP 621.

## Files to Read First

- `src/manifest.rs` — Single-file module with a parser function per language ecosystem.

## Current Status

Fully implemented for all 7 manifest formats. The config module calls `discover_from_manifests()` to get module names and source paths before falling back to extension-based scanning.

## Notes

- Balanced parenthesis extraction is used for Swift's `Package.swift` — it's a mini expression parser for the `.target(name: ..., path: ...)` syntax.
- Go module detection uses the last path segment of the module name (e.g., `github.com/user/repo` → `repo`) and probes for standard directory conventions (cmd, internal, pkg, api).
