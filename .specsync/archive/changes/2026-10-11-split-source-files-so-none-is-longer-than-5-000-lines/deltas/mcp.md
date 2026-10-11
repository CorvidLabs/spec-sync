## ADDED

### REQUIREMENT REQ-mcp-010
The MCP module SHALL keep the same behavior when its source is split across `include!` files, and none of those files SHALL exceed 5,000 lines.

Acceptance Criteria
- `src/mcp.rs` includes its parts in the original order.
- The parts are the previous `src/mcp.rs` bytes.
- No MCP source file exceeds 5,000 lines.
