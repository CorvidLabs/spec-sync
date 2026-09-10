---
change: teach-generated-agent-skills-the-specsync-6-0-check-is-product-lifecycle
artifact: docs
---

# Docs

Generated host docs (installer output, not the Astro site):

- `.claude/.codex/.cursor/.gemini` `SKILL.md`: `specsync check` is the product; SDD opt-in via
  `specsync change adopt`; 6.0 happy path with `--actor`, `--commit`, `--reviewer`, `ship`/`finalize`;
  v1 `start`/`verify`/`accept`/`archive` labeled recovery; slug ids; same-actor review kept.
- `create-change` / `check` / `audit` commands: same happy-path verbs; `--commit` for ship evidence.
- Canonical `specs/agents/agents.spec.md` purpose, changelog, and `REQ-agents-007`.
- Companion `specs/agents/context.md` and `tasks.md` current-status notes.

Site blog and hub copy live in CorvidLabs/site (PR #401), not this change.
