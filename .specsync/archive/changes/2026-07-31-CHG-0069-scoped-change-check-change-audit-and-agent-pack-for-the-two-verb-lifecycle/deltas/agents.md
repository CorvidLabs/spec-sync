## ADDED

### REQUIREMENT REQ-agents-check-audit-commands-001
`specsync agents install` SHALL generate `/specsync:check` and `/specsync:audit` command files for tools that support project-local commands, and skill prose SHALL teach the two-verb lifecycle model.

Acceptance Criteria
- Claude, Cursor, and Gemini receive check and audit command files.
- Skill content distinguishes `change check` (scoped) from `change audit` (actives + living specs).
- Template version advances so upgrades refresh generated artifacts.
