## ADDED

### REQUIREMENT REQ-cmd-new-002

The new command SHALL exclude recognized test files from auto-detected module sources.

Acceptance Criteria

- JavaScript-family `.test.*` and `.spec.*` files are omitted.
- Production files with configured or default source extensions remain included.
