---
change: teach-generated-agent-skills-the-specsync-6-0-check-is-product-lifecycle
artifact: design
---

# Design

No new agent tools, paths, or slash commands. Template rewrite plus version bump.

- Edit `SKILL_BODY` and the three command step templates in `src/agents.rs`.
- Bump `AGENT_ARTIFACT_TEMPLATE_VERSION` 4 to 5 so content-aware install overwrites the old
  SDD-first body on managed artifacts.
- Regenerate tracked host files from those templates (existing byte-parity tests).
- Add `REQ-agents-007` and a tracked-file pin test for the check-is-product sentences, matching
  the same-actor pin already in `tracked_skill_files_carry_the_current_same_actor_guidance`.
- Keep `hooks.rs` snippets unchanged in this change.
