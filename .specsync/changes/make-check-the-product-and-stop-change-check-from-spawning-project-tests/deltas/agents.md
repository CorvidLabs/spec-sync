## MODIFIED

### SPEC SECTION Purpose

Installs native, tool-owned verified-SDD skills for Claude Code, Cursor, Codex, and Gemini CLI.
Generated artifacts are tracked in `.specsync/agent-artifacts.json` so upgrades and uninstall can
distinguish exact managed bytes from user customization.

### REQUIREMENT REQ-agents-check-audit-commands-001

`specsync agents install` SHALL generate `/specsync:check` and `/specsync:audit` command files for tools that support project-local commands, and skill prose SHALL teach the two-verb lifecycle model.

Acceptance Criteria
- Claude, Cursor, and Gemini receive check and audit command files.
- Skill content distinguishes `change check` (scoped spec↔code sync) from `change audit` (actives + living specs).
- Template version advances so upgrades refresh generated artifacts.
- Generated `change check` skill and command files describe spec↔code sync, not project test commands.
