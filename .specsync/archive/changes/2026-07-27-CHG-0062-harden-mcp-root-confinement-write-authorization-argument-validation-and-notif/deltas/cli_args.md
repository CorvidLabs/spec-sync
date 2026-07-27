## ADDED

### REQUIREMENT REQ-cli-args-008

The shared CLI grammar SHALL expose explicit MCP write authorization.

Acceptance Criteria

- `specsync mcp --allow-write` enables mutating MCP tools.
- Omitting the flag keeps MCP read-only.
- Help describes the configured-root security boundary.

## MODIFIED

### SPEC SECTION Invariants

1. All global flags remain available around subcommands.
2. `--json` remains an alias for JSON output.
3. MCP is read-only unless the `Mcp` command's `--allow-write` flag is present.
4. Existing deterministic generation and verified SDD command grammar remains available.
