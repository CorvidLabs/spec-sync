# Lesson bundle — finish-the-same-actor-scoped-review-user-facing-copy-so-cli-adopting-and-generated-agent-skills-no-longer-demand-a

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Finish the same-actor scoped-review user-facing copy so CLI, ADOPTING, and generated agent skills no longer demand a second identity
- **Kind**: Feature
- **Specs**: cmd_change, agents
- **Paths**: src/commands/change.rs, src/agents.rs, docs/ADOPTING.md, .claude/skills/spec-sync/SKILL.md, .codex/skills/spec-sync/SKILL.md, .cursor/skills/spec-sync/SKILL.md, .gemini/skills/spec-sync/SKILL.md
- **Acceptance**: CLI next-action and ship-status copy uses --reviewer <human> and names scoped review, not a second identity.
- **Acceptance**: docs/ADOPTING.md no longer tells adopters to pass --reviewer "<someone else>".
- **Acceptance**: Tracked Claude, Codex, Cursor, and Gemini SKILL.md files match the current generated skill body: the reviewer MAY be the definition approver.

## Evidence

- Verification commit: `e6df4cb3f8ac488a1bc5680c03f067417f0746a2`
- Base commit: `8e658a1f122df2af13edea8a4be7f4a3365e4a9d`
- Verified by: `cargo test commands::change::`, `cargo test agents::`

## From the change's context.md

# Context

Kyntrin requested changes on CorvidLabs/spec-sync#749. The domain gate was already fixed: `change review` accepts the definition approver. The shipped *guidance* still contradicted that policy.

Leftovers named by the review:

- `docs/ADOPTING.md` example still used `--reviewer "<someone else>"`.
- Tracked `.claude` / `.codex` / `.cursor` / `.gemini` `SKILL.md` files still required an independent reviewer, even though `src/agents.rs` `SKILL_BODY` was already updated.
- `src/commands/change.rs` ship-status and next-action strings still said `independent review` and `--reviewer <other>`.

This follow-up is copy and generated-artifact regeneration only. GitHub remains merge authority. Distinct reviewers remain allowed. Do not invent a second identity.

## From the change's testing.md

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

## Where these lessons go

- `specs/cmd_change/context.md`
- `specs/agents/context.md`
