## MODIFIED

### REQUIREMENT REQ-agents-006

Generated SDD skill text SHALL describe scoped review as recordable by the same actor who approved the definition. It SHALL NOT instruct agents to invent a second human identity for solo work. Tracked Claude, Codex, Cursor, and Gemini `SKILL.md` files in this repository SHALL match that generated body.

Acceptance Criteria

- The generated skill's lifecycle steps tell the agent to record `change review` with the human who signed off, including when that human also recorded definition approval.
- The skill does not require picking a second identity solely to satisfy SpecSync.
- Tracked `.claude/skills/spec-sync/SKILL.md`, `.codex/skills/spec-sync/SKILL.md`, `.cursor/skills/spec-sync/SKILL.md`, and `.gemini/skills/spec-sync/SKILL.md` contain that same-actor guidance.
- A unit test fails if those tracked files drift from the current template on this point.
