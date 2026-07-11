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
