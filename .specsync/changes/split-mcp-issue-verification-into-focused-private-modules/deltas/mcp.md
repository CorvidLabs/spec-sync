## ADDED

### SPEC SECTION Internal Organization

MCP issue verification lives in the private `src/mcp/issues.rs` module, with its existing regressions in `src/mcp/issues/tests.rs`. Only the handler is visible to the parent module. Checked discovery, bounded content-free diagnostics, explicit GitHub configuration and inconclusive-error handling remain private and retain their existing behavior. Files are split by responsibility without expanding the public API or changing JSON output, security limits, authentication, or filesystem authority.
