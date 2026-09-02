## MODIFIED

### REQUIREMENT REQ-agents-check-audit-commands-001

`specsync agents install` SHALL generate `/specsync:check` and `/specsync:audit` command files for tools that support project-local commands, and skill prose SHALL teach the two-verb lifecycle model.

Acceptance Criteria
- Claude, Cursor, and Gemini receive check and audit command files.
- Skill content distinguishes `change check` (scoped spec↔code sync) from `change audit` (actives + living specs).
- Template version advances so upgrades refresh generated artifacts.
- Generated `change check` skill and command files describe spec↔code sync, not project test commands.
- Skill prose tells agents to clear context only when the `Handoff:` line says `safe`, and otherwise
  to do what `Before clearing:` names first; the template version advances so installed skills refresh.
