# Lesson bundle — teach-generated-agent-skills-the-specsync-6-0-check-is-product-lifecycle

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Teach generated agent skills the SpecSync 6.0 check-is-product lifecycle
- **Kind**: Documentation
- **Specs**: agents
- **Paths**: src/agents.rs, .claude/skills/spec-sync/SKILL.md, .codex/skills/spec-sync/SKILL.md, .cursor/skills/spec-sync/SKILL.md, .gemini/skills/spec-sync/SKILL.md, .claude/commands/specsync/create-change.md, .claude/commands/specsync/check.md, .claude/commands/specsync/audit.md, .cursor/commands/specsync-create-change.md, .cursor/commands/specsync-check.md, .cursor/commands/specsync-audit.md, .gemini/commands/specsync/create-change.toml, .gemini/commands/specsync/check.toml, .gemini/commands/specsync/audit.toml
- **Acceptance**: Generated Claude/Cursor/Codex/Gemini SKILL.md files lead with specsync check as the product and SDD opt-in via change adopt.
- **Acceptance**: Generated create-change/check/audit commands use 6.0 happy-path verbs: approve --actor, check --commit, review --reviewer, ship/finalize; v1 start/verify/accept/archive are labeled recovery only.
- **Acceptance**: AGENT_ARTIFACT_TEMPLATE_VERSION is 5 so specsync agents install refreshes stale host skills.
- **Acceptance**: cargo test agents:: passes.
- **Acceptance**: specsync check --spec agents passes.

## Evidence

- Verification commit: `9fe7f7f8234b3c50032a9f3083454d369c392e08`
- Base commit: `6f11e34ef65710dd8839bfaaa6e6bc9004d6a530`
- Verified by: `specsync check --spec agents`

## From the change's context.md

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

## From the change's design.md

# Design

No new agent tools, paths, or slash commands. Template rewrite plus version bump.

- Edit `SKILL_BODY` and the three command step templates in `src/agents.rs`.
- Bump `AGENT_ARTIFACT_TEMPLATE_VERSION` 4 to 5 so content-aware install overwrites the old
  SDD-first body on managed artifacts.
- Regenerate tracked host files from those templates (existing byte-parity tests).
- Add `REQ-agents-007` and a tracked-file pin test for the check-is-product sentences, matching
  the same-actor pin already in `tracked_skill_files_carry_the_current_same_actor_guidance`.
- Keep `hooks.rs` snippets unchanged in this change.

## From the change's testing.md

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

## Where these lessons go

- `specs/agents/context.md`
