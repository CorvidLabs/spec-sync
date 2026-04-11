---
spec: cmd_lifecycle.spec.md
---

## User Stories

- As a developer, I want to manage spec lifecycle statuses so that I can track spec maturity
- As a CI operator, I want transition guards to enforce quality gates before specs advance

## Acceptance Criteria

- All exported functions perform their documented purpose
- Transition validation prevents invalid status changes unless --force is used
- Guard evaluation checks min_score, required sections, and staleness
- Error conditions produce clear, actionable messages with exit code 1

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing
- Guard configuration is loaded from specsync.json lifecycle section
