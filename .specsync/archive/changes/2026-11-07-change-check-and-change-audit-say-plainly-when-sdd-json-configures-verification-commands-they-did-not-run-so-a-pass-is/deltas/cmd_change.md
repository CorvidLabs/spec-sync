## ADDED

### REQUIREMENT REQ-cmd-change-unrun-commands-001

Text `change check` (plain and `--commit`) and `change audit` SHALL disclose configured verification commands they did not run, and their successful JSON output SHALL expose the same commands as a top-level `configured_commands_not_run` list beside `verification` or `report`. The disclosure SHALL NOT change a command's verdict or execute a configured command (SHIP-1.c.3).

Acceptance Criteria
- With one or more configured commands, text prints one stderr line beginning `note: .specsync/sdd.json configures N verification command(s) that SpecSync did not run`, using singular `command` for N=1 and plural `commands` otherwise, and includes the words `to stop this note`; stdout is unchanged.
- JSON includes every configured command once, in policy order: `verification_commands`, `component_verification_commands` by module, then `strict_verification_commands`. With none configured, the key is present as `[]`, including when `verification` is null.
- Plain and committed text checks and text audit use the same note. JSON check and audit carry the same ordered list. `change check --commit --json` gains no success document, preserving #839.
- The disclosure never changes exit status and never spawns a configured command.
- SpecSync's own `.specsync/sdd.json` empties only `verification_commands`; all 13 component modules and all 3 strict commands remain configured and are disclosed rather than removed to silence the note.
