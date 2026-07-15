## ADDED

### REQUIREMENT REQ-cmd-scaffold-002

The add-spec scaffold SHALL exclude recognized test files from auto-detected module sources.

Acceptance Criteria

- JavaScript-family `.test.*` and `.spec.*` files are omitted.
- Production files with configured or default source extensions remain included.
