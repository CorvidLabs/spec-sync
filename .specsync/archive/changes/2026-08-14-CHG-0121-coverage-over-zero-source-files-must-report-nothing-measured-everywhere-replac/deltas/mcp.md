## ADDED

### REQUIREMENT REQ-mcp-004

MCP coverage surfaces SHALL report an unmeasured tree as unmeasured.

Acceptance Criteria
- `resource_coverage` and `tool_coverage` both emit `null` for a percentage that could not be computed.
- Neither surface reports a percentage a text run would decline to print.
