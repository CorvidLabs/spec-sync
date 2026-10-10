---
spec: cmd_rules.spec.md
---

## User Stories

- As a developer, I want to see all active validation rules (built-in and custom) so I can understand what checks run during `specsync check`
- As a CI operator, I want clear output showing rule severity and filter criteria so I can configure rules appropriate for my project

## Acceptance Criteria

- All exported functions perform their documented purpose
- Built-in rules are always listed with their active/off status
- Custom rules display name, type, severity, and filter criteria when defined
- Error conditions produce clear, actionable messages
- Module follows the project's established patterns for config loading and output formatting

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing
- Read-only command: must never modify config or spec files

## Out of Scope

- GUI or web interface
- Interactive prompts (except wizard module)
- Rule editing or creation (this command is display-only)

### REQ-cmd-rules-001

The rules command SHALL display effective built-in and declarative validation rules from the loaded configuration without mutating project state.

Acceptance Criteria
- All exported functions perform their documented purpose
- Built-in rules are always listed with their active/off status
- Custom rules display name, type, severity, and filter criteria when defined
- Error conditions produce clear, actionable messages
- Module follows the project's established patterns for config loading and output formatting

### REQ-cmd-rules-002

The rules command SHALL name the project's principles file beside the spec-shape rules. It SHALL say that a lesson becomes a standing rule only when a person writes it into that file. When no principles file is configured, it SHALL say so. The command SHALL remain read-only.

Acceptance Criteria
- A configured `principles_file` is printed with its path.
- The output says SpecSync does not copy lessons into that file.
- An unset principles file prints that none is configured.
- Built-in rule listing is unchanged.
- The command does not write the principles file or any spec.

