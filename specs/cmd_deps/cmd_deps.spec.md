---
module: cmd_deps
version: 3
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

Implements the `specsync deps` command. Validates cross-module dependency declarations and optionally renders the dependency graph as Mermaid or Graphviz DOT diagrams. Under `--strict`, dependency warnings (undeclared imports) are treated as failures so CI can gate on them.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_deps` | `root: &Path, strict: bool, format: OutputFormat, mermaid: bool, dot: bool` | `()` | Validate dependency graph; optionally output as Mermaid or DOT. Under `strict`, dependency warnings become failures |

## Invariants

1. Core validation delegates to `deps::validate_deps()`
2. Private helpers `render_mermaid()` and `render_dot()` generate diagram syntax
3. Exits 1 if dependency errors found (cycles, missing deps)
4. Under `--strict`, a non-empty warning set (undeclared imports) also forces exit 1, after the report is printed in the requested format
5. The `--strict` failure note is a human diagnostic printed to stderr and is suppressed in JSON mode — JSON stays fully machine-readable (the failing warnings are in the `warnings` array and the non-zero exit code carries the verdict)

## Behavioral Examples

### Scenario: Mermaid output

- **Given** `--mermaid` flag set, clean dep graph
- **When** `cmd_deps` runs
- **Then** outputs valid Mermaid flowchart syntax

### Scenario: Cycle detected

- **Given** A depends on B, B depends on A
- **When** `cmd_deps` runs
- **Then** prints cycle error and exits 1

### Scenario: Strict mode gates undeclared-import warnings

- **Given** a module imports another that is not in its `depends_on`, and `--strict` is set
- **When** `cmd_deps` runs (no dependency errors, only warnings)
- **Then** prints the report, notes on stderr that warnings are treated as errors, and exits 1

### Scenario: Strict failure in JSON mode

- **Given** the same undeclared-import warning under `--strict --format json`
- **When** `cmd_deps` runs
- **Then** stdout is the JSON report (with the warning in `warnings`), the human "treated as errors" note is **not** emitted, and the process exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Circular dependency | Error printed, exits 1 |
| Missing dependency spec | Error printed, exits 1 |
| Undeclared import under `--strict` | Warning printed; stderr note (non-JSON only); exits 1 |
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
| 2026-07-03 | v2: `cmd_deps` gained a `strict` parameter (#304) — undeclared-import warnings now force exit 1 under `--strict`. Documented the new signature, the strict exit-code invariant, and that the strict stderr note is suppressed in JSON mode (follow-up to #304). |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
