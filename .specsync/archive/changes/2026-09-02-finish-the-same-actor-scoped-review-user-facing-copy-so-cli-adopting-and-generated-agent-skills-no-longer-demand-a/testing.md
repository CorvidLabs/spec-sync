---
change: finish-the-same-actor-scoped-review-user-facing-copy-so-cli-adopting-and-generated-agent-skills-no-longer-demand-a
artifact: testing
---

# Testing

- `cargo test commands::change::`
- `cargo test agents::`
- Tracked SKILL.md files contain the same-actor sentence from `SKILL_BODY`.
- `rg` over CLI copy, ADOPTING.md, and tracked skills finds no `--reviewer <other>` and no `--reviewer "<someone else>"`.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-cmd-change-015 | `src/commands/change.rs` ship-status and next-action strings; `docs/ADOPTING.md` |
| REQ-agents-006 | Tracked `.claude` / `.codex` / `.cursor` / `.gemini` `SKILL.md`; `src/agents.rs` parity test |
