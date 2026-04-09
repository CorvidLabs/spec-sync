---
module: cmd_view
version: 1
status: stable
files:
  - src/commands/view.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/validator/validator.spec.md
  - specs/view/view.spec.md
---

# Cmd View

## Purpose

Implements the `specsync view` command. Renders specs filtered by role (dev, qa, product, agent), showing only relevant sections.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_view` | `root: &Path, role: &str, spec_filter: Option<&str>` | `()` | Render specs filtered for a specific role |

## Invariants

1. Delegates to `view::view_spec()` for role filtering
2. Exits 1 if no specs found
3. Errors per-file are printed but processing continues

## Behavioral Examples

### Scenario: Dev view

- **Given** `specsync view --role dev --spec auth`
- **When** `cmd_view` runs
- **Then** renders auth spec with dev-relevant sections only

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No specs found | Exits 1 |
| Spec read error | Error printed, continues |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| validator | `find_spec_files` |
| view | `view_spec` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync view` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
