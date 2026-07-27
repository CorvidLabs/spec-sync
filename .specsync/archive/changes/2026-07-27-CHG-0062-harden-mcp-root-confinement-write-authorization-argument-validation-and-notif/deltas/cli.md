## ADDED

### REQUIREMENT REQ-cli-005

The root CLI dispatcher SHALL preserve MCP write authorization and fail closed when the configured
server root cannot be resolved.

Acceptance Criteria

- The dispatcher forwards the parsed `allow_write` capability to `run_mcp_server` without changing
  its default.
- MCP startup errors are printed to stderr and exit with usage status 2.
- No MCP request is read when server-root initialization fails.

## MODIFIED

### SPEC SECTION Invariants

1. When no subcommand is given, `check` runs by default.
2. `--root` defaults to the current working directory; the path is validated as an existing
   directory and canonicalized, otherwise the process exits 2.
3. `--strict` causes warnings to produce a non-zero exit code.
4. `--require-coverage N` causes exit 1 if file coverage percent is below N.
5. `--json` switches all output to machine-readable JSON without ANSI colors.
6. Initialization and registry creation remain idempotent and preserve existing configuration.
7. Generation, scoring, resolution, hooks, lifecycle, and reporting commands delegate policy to
   their focused modules.
8. Inherited verification context rejects recursive check/change/lifecycle dispatch before handler
   execution.
9. MCP dispatch forwards the parsed write capability unchanged; read-only remains the default.
10. MCP server-root initialization failures are printed to stderr and exit 2 before request input is
    processed.
