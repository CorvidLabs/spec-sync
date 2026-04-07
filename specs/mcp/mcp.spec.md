---
module: mcp
version: 1
status: stable
files:
  - src/mcp.rs
db_tables: []
tracks: [113]
depends_on:
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/config/config.spec.md
  - specs/scoring/scoring.spec.md
  - specs/generator/generator.spec.md
  - specs/ai/ai.spec.md
---

# Mcp

## Purpose

Model Context Protocol (MCP) server for AI agent integration. Implements JSON-RPC 2.0 over stdio, exposing spec-sync functionality as tools callable from Claude Code, Cursor, Windsurf, and other MCP-compatible agents.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `run_mcp_server` | `root: &Path` | `()` | Run the MCP server on stdio, processing JSON-RPC 2.0 requests |

## Invariants

1. Protocol version is "2024-11-05"
2. Server reports capabilities with `tools: {}` — no prompts or resources
3. Six tools are exposed: `specsync_check`, `specsync_coverage`, `specsync_generate`, `specsync_list_specs`, `specsync_init`, `specsync_score`
4. All tool responses use `content: [{ type: "text", text: "..." }]` format
5. Errors are returned as `isError: true` in the result, not as JSON-RPC errors (except parse errors and method-not-found)
6. Notifications (requests without `id`) receive no response
7. `ping` method returns an empty result object
8. Each tool accepts an optional `root` parameter to override the default project root

## Behavioral Examples

### Scenario: Initialize MCP session

- **Given** a client sends `{"jsonrpc":"2.0","id":1,"method":"initialize"}`
- **When** the server processes the request
- **Then** responds with protocol version, capabilities, and server info

### Scenario: Call specsync_check tool

- **Given** a client sends a `tools/call` request with `name: "specsync_check"`
- **When** the server processes the request
- **Then** responds with validation results including passed/failed status, errors, and warnings

### Scenario: Unknown method

- **Given** a client sends a request with `method: "unknown/method"` and an `id`
- **When** the server processes the request
- **Then** responds with JSON-RPC error code -32601 "Method not found"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Malformed JSON input | JSON-RPC error -32700 "Parse error" |
| Unknown method with id | JSON-RPC error -32601 "Method not found" |
| Unknown tool name | Tool error: "Unknown tool: {name}" |
| No spec files found | Tool error with suggestion to run `specsync generate` |
| stdin EOF | Server exits gracefully |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config`, `detect_source_dirs` |
| validator | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names` |
| generator | `generate_specs_for_unspecced_modules_paths` |
| scoring | `score_spec`, `compute_project_score` |
| ai | `resolve_ai_provider` |
| parser | `parse_frontmatter` |
| types | `SpecSyncConfig` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `run_mcp_server` (via `mcp` subcommand) |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
