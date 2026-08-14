## ADDED

### REQUIREMENT REQ-commands-009

No command SHALL report a verdict derived from configuration that failed to load.

Acceptance Criteria
- A command that reads specs refuses to run when the configuration records a load failure, and names the file.
- The refusal states how to proceed: fix the file, or remove it to use the built-in defaults deliberately.
- A project with a valid configuration, and a project with none, are both unaffected.
- The refusal is applied once at the shared entry point, so no command can omit it.
