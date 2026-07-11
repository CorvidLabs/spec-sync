---
module: cmd_change
version: 1
status: active
files:
  - src/commands/change.rs
db_tables: []
tracks: []
depends_on:
  - specs/change/change.spec.md
  - specs/types/types.spec.md
---

# Cmd Change

## Purpose

Exposes the verified SDD lifecycle through equivalent human-readable and structured JSON commands under `specsync change`.

## Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Status and interviews provide a concrete next action.

## Public API

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `cmd_change` | `root: &Path, action: ChangeAction, format: OutputFormat` | `()` | Dispatch every change lifecycle command and render text or JSON |

## Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` fails on any lifecycle error.

## Behavioral Examples

### Scenario: Agent creates a change

- **Given** `specsync --json change new "Add passkeys"`
- **When** creation succeeds
- **Then** JSON includes the record, gate summary, and deterministic questions

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown change type | Descriptive error and exit 1 |
| Invalid transition | Current and expected states plus exit 1 |

## Dependencies

| Module | What is used |
|--------|-------------|
| change | Lifecycle operations and projections |
| types | Output format |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | Initial 5.0 change command |
