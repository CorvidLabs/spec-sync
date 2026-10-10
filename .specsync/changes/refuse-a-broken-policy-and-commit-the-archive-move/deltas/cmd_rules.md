## ADDED

### REQUIREMENT REQ-cmd-rules-003
When `principles_file` is set and that path is not a file, `specsync rules` SHALL warn that the file is named in `.specsync/sdd.json` but is not there. The summary SHALL say the file is missing and SHALL say to create it or clear `principles_file`. The command SHALL exit 0 and SHALL NOT create the file.

Acceptance Criteria
- The summary names the configured path, says missing, and says to create it.
- Stderr warns that the file is named but is not there.
- An unset principles file still says none is configured.
- A policy that cannot be parsed still fails the command.
