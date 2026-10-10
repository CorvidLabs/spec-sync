# Lesson bundle — tell-agents-to-record-lessons-in-the-module-3md-file

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Tell agents to record lessons in the module 3md file
- **Kind**: BugFix
- **Specs**: agents, hooks
- **Paths**: src/agents.rs, src/hooks.rs, AGENTS.md, specs/, .claude/skills/spec-sync/SKILL.md, .codex/skills/spec-sync/SKILL.md, .cursor/skills/spec-sync/SKILL.md, .gemini/skills/spec-sync/SKILL.md
- **Acceptance**: Agents are told to record a new lesson on a later plane of module.3md when that file exists.
- **Acceptance**: Each module spec changelog records that the 3md file is beside the spec.

## Evidence

- Verification commit: `afad988a8bf9d4835f3f491a5ebece03bf7195d4`
- Base commit: `fc0c668270c42efa6ead1a9f759c1ba61ac77904`
- Verified by: `specsync check --spec agents --spec ai --spec hooks`

## From the change's context.md

# Context

The module 3md files are now the lesson source. The agent instructions still told people to write only in `context.md`, so the next lesson would never reach a briefing.

The skill, the hook snippets, and AGENTS.md now say to record a new lesson on a later plane of `<module>.3md` when that file exists. Each module spec changelog records the file.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-agents-008 | `tracked_skill_files_teach_check_is_the_product` requires each tracked SKILL.md to contain `later plane`. |
| REQ-hooks-004 | The hook snippets in `src/hooks.rs` tell the agent to record a lesson on a later plane when `<module>.3md` exists. |

The four tracked skill files and AGENTS.md use that same instruction.

## Where these lessons go

- `specs/agents/agents.3md`
- `specs/hooks/hooks.3md`
