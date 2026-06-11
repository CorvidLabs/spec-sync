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
  - specs/ai/ai.spec.md
  - specs/generator/generator.spec.md
  - specs/output/output.spec.md
  - specs/ignore/ignore.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Generate

## Purpose

Implements the `specsync generate` command. Scaffolds spec files for unspecced modules using templates or AI-assisted generation when `--provider` is passed.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_generate` | `root, strict, enforcement, require_coverage, format, provider` | `()` | Generate specs for uncovered modules; optionally use AI for content |

## Invariants

1. Without `--provider`, generates from templates only
2. With `--provider`, resolves AI provider and generates from source code
3. Re-runs validation after generation to verify new specs
4. Exits 1 if AI provider resolution fails
5. If any AI generation fails (template fallback used), the failures are re-printed prominently on stderr after the check report and the command exits 1; JSON output includes an `ai_errors` array

## Behavioral Examples

### Scenario: AI-assisted generation

- **Given** `--provider claude` set, 3 unspecced modules
- **When** `cmd_generate` runs
- **Then** generates 3 AI-populated specs

## Error Cases

| Condition | Behavior |
|-----------|----------|
| AI provider not found | Exits 1 |
| AI fails for one module | Error printed, continues, then exits 1 with the failures summarized last on stderr |
| All modules already specced | Prints "all covered" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns`, `run_validation`, `exit_with_status` |
| ai | `resolve_ai_provider`, `generate_spec_with_ai` |
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
