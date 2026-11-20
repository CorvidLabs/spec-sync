## ADDED

### REQUIREMENT REQ-mcp-014

The MCP issues tool SHALL find spec files only through the confined spec walk, so a `specs` link that leaves the project is refused, not walked.

Acceptance Criteria
- The MCP issues walk uses `find_spec_files_checked` and still reports a non-regular spec path as inconclusive.
