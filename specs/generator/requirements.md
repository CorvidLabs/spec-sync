---
spec: generator.spec.md
---

## User Stories

- As a developer, I want reproducible local scaffolds for every uncovered module.
- As a team, I want custom templates and existing files preserved.
- As a coding-agent user, I want standard companion files ready for enrichment.

## Constraints

- Generated files must be valid inputs to `specsync check`.
- Source discovery excludes tests and remains deterministic.
