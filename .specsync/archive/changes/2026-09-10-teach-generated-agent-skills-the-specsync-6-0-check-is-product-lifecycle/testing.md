---
change: teach-generated-agent-skills-the-specsync-6-0-check-is-product-lifecycle
artifact: testing
---

# Testing

- Unit: `cargo test agents::` — 42 passed, including
  `tracked_skill_files_teach_check_is_the_product` and
  `tracked_skill_files_carry_the_current_same_actor_guidance`.
- Spec: `specsync check --strict agents` — pass.
- Installer: `specsync agents install` in this tree refreshed 16 managed artifacts to
  `template_version` 5.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-agents-007 | Tracked Claude/Codex/Cursor/Gemini `SKILL.md` files contain `` `specsync check` is the product ``, `specsync change adopt`, `approve --actor`, `check --commit`, `review --reviewer`, and `v1 recovery`. Pin: `tracked_skill_files_teach_check_is_the_product`. `AGENT_ARTIFACT_TEMPLATE_VERSION` is 5; `.specsync/agent-artifacts.json` records template_version 5 on all 16 artifacts. |
