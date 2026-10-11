## ADDED

### REQUIREMENT REQ-github-022
The GitHub source scan SHALL read every MCP source file it checks for a `gh` process, including the files included by `src/mcp.rs`.

Acceptance Criteria
- The scan includes `src/mcp.rs`, `src/mcp/server_name.rs`, and `src/mcp/publish_staged_file_with_verification_hooks.rs`.
- A `gh` process constructed in an MCP part still fails the scan.
