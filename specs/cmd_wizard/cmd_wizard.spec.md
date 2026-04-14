---
module: cmd_wizard
version: 1
status: stable
files:
  - src/commands/wizard.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/generator/generator.spec.md
---

# Cmd Wizard

## Purpose

Implements the `specsync wizard` command — an interactive TUI wizard for creating new specs step by step. Uses `dialoguer` prompts for module name, purpose, template type, status, source files, and dependencies. Supports template-specific sections (API endpoint, data model, utility, UI component) and shows a preview before writing.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_wizard` | `root: &Path` | `()` | Launch interactive spec creation wizard with prompts for all spec fields |

## Invariants

1. User can cancel at any prompt — exits cleanly with code 0
2. Module name must be non-empty
3. Refuses to overwrite existing spec (exits 1 if spec dir already exists)
4. Source file auto-detection scans `source_dirs` for files matching the module name/directory
5. Template types: Generic, API Endpoint, Data Model, Utility, UI Component — each adds template-specific sections
6. Companion files (tasks.md, context.md, requirements.md, testing.md) are always generated; design.md is generated only when `companions.design` is enabled in config
7. Shows a full preview of the spec before asking for write confirmation

## Behavioral Examples

### Scenario: Create API endpoint spec

- **Given** user selects "API Endpoint" template
- **When** wizard generates spec content
- **Then** includes Endpoints table section with Method, Path, Description columns

### Scenario: Auto-detect source files

- **Given** module name "auth", `src/auth.rs` exists
- **When** wizard runs source detection
- **Then** pre-fills source files with `src/auth.rs`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty module name entered | Exits with code 1 |
| Spec directory already exists | Prints error and exits 1 |
| User cancels at confirmation | Exits cleanly with code 0 |
| Directory creation fails | Exits with code 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| generator | Companion file generation |
| dialoguer | `Input`, `Select`, `Confirm` prompts |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync wizard` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-04-13 | Document testing.md and conditional design.md in companion generation |
