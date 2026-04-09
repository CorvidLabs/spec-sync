---
module: cmd_deps
version: 1
status: stable
files:
  - src/commands/deps.rs
db_tables: []
tracks: []
depends_on:
  - specs/deps/deps.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Deps

## Purpose

Implements the `specsync deps` command. Validates cross-module dependency declarations and optionally renders the dependency graph as Mermaid or Graphviz DOT diagrams.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_deps` | `root: &Path, format: OutputFormat, mermaid: bool, dot: bool` | `()` | Validate dependency graph; optionally output as Mermaid or DOT |

## Invariants

1. Core validation delegates to `deps::validate_deps()`
2. Private helpers `render_mermaid()` and `render_dot()` generate diagram syntax
3. Exits 1 if dependency errors found (cycles, missing deps)

## Behavioral Examples

### Scenario: Mermaid output

- **Given** `--mermaid` flag set, clean dep graph
- **When** `cmd_deps` runs
- **Then** outputs valid Mermaid flowchart syntax

### Scenario: Cycle detected

- **Given** A depends on B, B depends on A
- **When** `cmd_deps` runs
- **Then** prints cycle error and exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Circular dependency | Error printed, exits 1 |
| Missing dependency spec | Error printed, exits 1 |
| Empty dep graph | Prints hint about `depends_on` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| deps | `validate_deps` |
| config | `load_config` |
| types | `OutputFormat` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync deps` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
