## ADDED

### REQUIREMENT REQ-agents-008
Generated skill text SHALL tell an agent that when `<module>.3md` exists beside the spec, a new lesson goes on a later plane of that file. The first plane SHALL be described as context. The skill SHALL NOT tell the agent that `context.md` is the only place a lesson can be recorded while that file parses.

Acceptance Criteria
- Each tracked SKILL.md contains `later plane`.
- The skill still says to update `context.md` when the 3md file is absent.
- `tracked_skill_files_teach_check_is_the_product` fails if a tracked skill drops `later plane`.
