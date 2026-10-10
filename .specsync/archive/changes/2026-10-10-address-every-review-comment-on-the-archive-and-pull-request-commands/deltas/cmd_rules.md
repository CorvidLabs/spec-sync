## MODIFIED

### REQUIREMENT REQ-cmd-rules-002
The rules command SHALL name the project's principles file beside the spec-shape rules. It SHALL say that a lesson becomes a standing rule only when a person writes it into that file. When no principles file is configured, it SHALL say so. The command SHALL remain read-only.

Acceptance Criteria
- A configured `principles_file` is printed with its path.
- The output says SpecSync does not copy lessons into that file.
- An unset principles file prints that none is configured.
- A malformed or unreadable `.specsync/sdd.json` fails the command with the policy error instead of claiming that no principles file is configured.
- Built-in rule listing is unchanged.
- The command does not write the principles file or any spec.
