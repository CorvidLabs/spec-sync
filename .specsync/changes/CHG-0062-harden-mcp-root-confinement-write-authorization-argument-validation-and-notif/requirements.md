---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: requirements
---

# Requirements

### REQ-mcp-002

The MCP server SHALL confine every filesystem operation to its configured project root and SHALL
expose mutating tools only when the operator explicitly enables writes.

Acceptance Criteria

- `specsync mcp` is read-only by default and omits mutating tools from `tools/list`.
- A direct call to a mutating tool in read-only mode is rejected before tool execution.
- `specsync mcp --allow-write` exposes mutation, but mutating tools always use the configured
  server root and reject a per-call root override.
- Read-only tool roots must resolve to an existing canonical descendant of the server root.
- Configuration/metadata/cache files, configured paths, manifest workspace and autodetection paths,
  dependency references, module names/files, spec file mappings, generated destinations, and nested
  symlink targets must remain within the canonical root.
- Recursive checks must use cumulative deterministic budgets across configured paths and spec
  mappings, canonicalize only symlinks, and honor ignored or configured-excluded directories.
- Absolute escapes, parent traversal, nonexistent outside roots, configured-path escapes, and
  symlink escapes are rejected before downstream filesystem access.
- Rejected calls do not create, overwrite, or remove files outside the server root.

### REQ-mcp-003

The MCP server SHALL validate JSON-RPC tool arguments exactly and SHALL obey notification response
semantics.

Acceptance Criteria

- Advertised tool schemas reject unknown properties and require the documented value types.
- Malformed `tools/call` parameters return JSON-RPC `-32602` without executing a tool.
- Tool execution failures remain MCP tool results with `isError: true`.
- Every request without an `id`, including an unknown method, emits no response.
- Mutation requires an acknowledged request ID; mutation notifications never write.
- JSON-RPC lines larger than 1 MiB are drained and rejected with `-32700`; the next line remains an
  independent request boundary.

### REQ-cli-args-008

The shared CLI grammar SHALL expose explicit MCP write authorization.

Acceptance Criteria

- `specsync mcp --allow-write` parses as write-enabled MCP server mode.
- Omitting `--allow-write` selects read-only mode.
- Help text explains that the flag exposes mutating MCP tools within the configured root.

### REQ-cli-005

The root CLI dispatcher SHALL preserve MCP write authorization and fail closed when the configured
server root cannot be resolved.

Acceptance Criteria

- The dispatcher forwards the parsed `allow_write` capability to `run_mcp_server` without changing
  its default.
- MCP startup errors are printed to stderr and exit with usage status 2.
- No MCP request is read when server-root initialization fails.
