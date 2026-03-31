---
spec: mcp.spec.md
---

## User Stories

- As a Claude Code user, I want spec-sync available as an MCP server so that I can check, generate, and score specs directly from my AI assistant
- As a Cursor user, I want MCP tools for spec-sync so that spec validation is integrated into my AI-powered editing workflow
- As an AI agent developer, I want programmatic access to spec-sync over JSON-RPC so that I can build spec validation into automated workflows
- As a developer, I want the MCP server to run over stdio so that it works with any MCP-compatible client without network configuration

## Acceptance Criteria

- Implements JSON-RPC 2.0 over stdio
- Protocol version "2024-11-05" returned in initialize response
- Six tools exposed: specsync_check, specsync_coverage, specsync_generate, specsync_list_specs, specsync_init, specsync_score
- Tool errors returned as `isError: true` in result content, not as JSON-RPC error objects (except parse/method-not-found)
- Malformed JSON returns JSON-RPC error -32700 "Parse error"
- Unknown method returns JSON-RPC error -32601 "Method not found"
- Unknown tool name returns tool-level error "Unknown tool: {name}"
- Notifications (requests without id) receive no response
- `ping` method returns empty result
- Each tool accepts optional `root` parameter to override project directory
- stdin EOF triggers graceful exit

## Constraints

- Must conform to the MCP specification — no custom protocol extensions
- All output must be valid JSON (no ANSI colors, no stderr mixing into protocol)
- Must be stateless — each tool call is independent

## Out of Scope

- MCP resources or prompts (only tools are implemented)
- HTTP/SSE transport (stdio only)
- Authentication or authorization
- Streaming partial results during long operations
