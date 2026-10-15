---
module: cmd_generate
version: 9
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

5. Checked manifest discovery must succeed before generation; malformed Gradle settings are
   inconclusive, produce no files, and exit 1.

6. Batch requests recognize existing scoped specs using the same leaf filename as the writer; reruns preserve spec and companion bytes.

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
| Malformed Gradle settings prevent coverage discovery | Exits 1 before generation; JSON remains valid with `valid: false`, `inconclusive: true`, an explicit error, and `generated: []` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns`, `run_validation`, `exit_with_status` |
| generator | `generate_specs_for_unspecced_modules_retained`, shared `module_spec_filename` leaf naming |
| output | `print_summary`, `print_coverage_line` |
| ignore | `IgnoreRules::load` |
| validator | `compute_coverage_checked` |
| types | `OutputFormat`, `EnforcementMode` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync generate` |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-10 | Add `cmd_generate.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-07-22 | v4: fail closed before generation when malformed Gradle/manifest discovery makes coverage inconclusive |
| 2026-04-09 | Initial spec |
| 2026-06-11 | v2: Exit non-zero when AI generation fails, with the errors re-printed last on stderr and `ai_errors` in JSON output |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-09-09 | close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving: Close the SpecSync 6.0.0 P1 release defects found in overnight proving |
| 2026-10-15 | complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1: Complete exact Cargo source handling and scoped batch recognition for SpecSync 6.1.1 |
