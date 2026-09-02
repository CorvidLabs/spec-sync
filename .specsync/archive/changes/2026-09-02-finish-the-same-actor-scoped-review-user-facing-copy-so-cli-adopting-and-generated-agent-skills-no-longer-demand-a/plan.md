---
change: finish-the-same-actor-scoped-review-user-facing-copy-so-cli-adopting-and-generated-agent-skills-no-longer-demand-a
artifact: plan
---

# Plan

1. Replace remaining `--reviewer <other>` / `independent review` next-action strings in `src/commands/change.rs` with `--reviewer <human>` and `scoped review`.
2. Change the ADOPTING.md happy-path example to `--reviewer "<human>"` and reword the squash section so it names scoped review without requiring a distinct actor.
3. Regenerate the four tracked `SKILL.md` files from current `SKILL_BODY` via `specsync agents install`.
4. Add a unit test that the tracked skill files contain the same-actor guidance so they cannot drift from the template again.
5. `cargo test commands::change::` and `cargo test agents::`.
