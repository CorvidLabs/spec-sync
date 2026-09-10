---
change: teach-generated-agent-skills-the-specsync-6-0-check-is-product-lifecycle
artifact: context
---

# Context

SpecSync 6.0.0 shipped 2026-09-09. Generated host skills and slash commands still taught the
pre-ship SDD-first story: every meaningful edit starts a change, `approve` without `--actor`,
`change check` without `--commit`, `finalize` without the review/ship pairing, and no
`change adopt` on-switch.

`SKILL_BODY` in `src/agents.rs` is the single source for Claude, Cursor, Codex, and Gemini
`SKILL.md`. Command bodies live in `CREATE_CHANGE_STEPS_MD`, `CHECK_CHANGE_STEPS_MD`, and
`AUDIT_CHANGE_STEPS_MD`. `AGENT_ARTIFACT_TEMPLATE_VERSION` must bump so `specsync agents install`
refreshes managed artifacts.

Out of scope: unifying `hooks.rs` snippets with `SKILL_BODY` (already recorded as a deliberate
split in `specs/agents/context.md`); adding slash commands for `ship`/`status`/`adopt`/`review`;
the shared catalog skill in the separate `skills` repo; committing untracked `.agents/` grok copies.
