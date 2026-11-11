## ADDED

### REQUIREMENT REQ-cmd-generate-004

`generate`, in every mode, SHALL write specs whose `files:` lists contain only files no existing spec claims.

Acceptance Criteria
- `generate --batch` and the MCP generate tool produce no double-mapped files.
- A module with no unowned candidate files is skipped with the existing no-files outcome.
