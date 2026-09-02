---
change: finish-the-same-actor-scoped-review-user-facing-copy-so-cli-adopting-and-generated-agent-skills-no-longer-demand-a
artifact: context
---

# Context

Kyntrin requested changes on CorvidLabs/spec-sync#749. The domain gate was already fixed: `change review` accepts the definition approver. The shipped *guidance* still contradicted that policy.

Leftovers named by the review:

- `docs/ADOPTING.md` example still used `--reviewer "<someone else>"`.
- Tracked `.claude` / `.codex` / `.cursor` / `.gemini` `SKILL.md` files still required an independent reviewer, even though `src/agents.rs` `SKILL_BODY` was already updated.
- `src/commands/change.rs` ship-status and next-action strings still said `independent review` and `--reviewer <other>`.

This follow-up is copy and generated-artifact regeneration only. GitHub remains merge authority. Distinct reviewers remain allowed. Do not invent a second identity.
