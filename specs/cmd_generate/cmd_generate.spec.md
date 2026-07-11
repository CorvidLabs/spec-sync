---
module: cmd_generate
version: 2
status: stable
files:
  - src/commands/generate.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/generator/generator.spec.md
  - specs/output/output.spec.md
  - specs/ignore/ignore.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Generate

## Purpose

Implements deterministic `specsync generate` scaffolding for unspecced modules using local templates.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_generate` | `root, strict, enforcement, require_coverage, format, uncovered, batch` | `()` | Deterministically generate specs for uncovered modules or a selected batch |

## Invariants

1. Generation always uses local deterministic templates and performs no inference or shell execution
2. Re-runs validation after generation to verify new specs
3. Batch selection reports generated, already-specced, and unknown modules deterministically
4. JSON output has no provider-specific fields

## Behavioral Examples

### Scenario: Deterministic generation

- **Given** 3 unspecced modules
- **When** `cmd_generate` runs
- **Then** generates 3 local template specs with companions

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot create or write a spec | Reports the module failure and continues safely |
| All modules already specced | Prints "all covered" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns`, `run_validation`, `exit_with_status` |
| generator | `generate_spec_template` |
| output | `print_summary`, `print_coverage_line` |
| ignore | `IgnoreRules::load` |
| validator | `compute_coverage` |
| types | `OutputFormat`, `EnforcementMode` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync generate` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-06-11 | v2: Exit non-zero when AI generation fails, with the errors re-printed last on stderr and `ai_errors` in JSON output |
