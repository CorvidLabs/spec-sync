---
change: tell-agents-to-record-lessons-in-the-module-3md-file
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-agents-008 | `tracked_skill_files_teach_check_is_the_product` requires each tracked SKILL.md to contain `later plane`. |
| REQ-hooks-004 | The hook snippets in `src/hooks.rs` tell the agent to record a lesson on a later plane when `<module>.3md` exists. |

The four tracked skill files and AGENTS.md use that same instruction.
