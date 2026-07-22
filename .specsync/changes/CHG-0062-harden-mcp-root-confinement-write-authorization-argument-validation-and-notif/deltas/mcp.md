## ADDED

### REQUIREMENT REQ-mcp-002

The MCP server SHALL confine every filesystem operation to its configured project root and SHALL
expose mutating tools only when the operator explicitly enables writes.

Acceptance Criteria

- Default MCP mode omits mutating tools.
- Direct mutator calls in default mode fail before execution.
- Write mode is explicit and mutating tools cannot override the configured root.
- Read roots are existing canonical descendants of the configured root.
- Configuration/metadata/cache files, manifest/autodetection paths, dependency references, module
  names/files, spec mappings, and nested symlink targets are confined before downstream filesystem
  access.
- Recursive confinement uses cumulative deterministic budgets across configured paths and spec
  mappings and honors ignored/configured exclusions.
- Traversal, configured-path, and symlink escapes fail before filesystem access.

### REQUIREMENT REQ-mcp-003

The MCP server SHALL validate JSON-RPC tool arguments exactly and SHALL obey notification response
semantics.

Acceptance Criteria

- Unknown keys and wrong argument types return `-32602` without tool execution.
- Tool-domain failures remain `isError: true` results.
- Every notification, including unknown methods, emits no response and cannot mutate files.
- JSON-RPC lines larger than 1 MiB are drained and rejected with `-32700` before parsing; the next
  line remains independently processable.

## MODIFIED

### SPEC SECTION Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `run_mcp_server` | `root: &Path, allow_write: bool` | `Result<(), String>` | Run the confined MCP server; mutating tools are exposed only when writes are explicitly enabled and root resolution failures are reported |

### SPEC SECTION Invariants

1. Protocol version is "2024-11-05".
2. Server reports tools and resources capabilities.
3. Read-only mode exposes five non-mutating tools; write mode additionally exposes
   `specsync_generate` and `specsync_init`.
4. All filesystem operations remain within the canonical configured root.
5. Read tools may select only an existing canonical descendant root.
6. Mutating tools require write mode, reject root overrides, and use the configured root.
7. Tool argument schemas and runtime validation reject unknown properties and wrong types.
8. Tool-domain errors use `isError`; JSON-RPC shape errors use protocol error objects.
9. Every notification, including unknown methods, receives no response and cannot mutate state.
10. Config/metadata/cache files and paths, manifest/autodetection paths, dependency references,
    module names/files, spec mappings, nested symlinks, and write destinations are validated against
    the canonical root before use.
11. Recursive checks canonicalize only symlinks, honor ignored/configured exclusions, and share
    deterministic cumulative bounds across configured paths and spec mappings.
12. JSON-RPC input lines are bounded to 1 MiB and oversized lines are drained before the next request.
13. Resources and deterministic generation behavior remain compatible when authorized.
