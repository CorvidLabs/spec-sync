## ADDED

### SPEC SECTION Integration Test Organization

The registered `tests/integration/mcp.rs` module is a small facade for private topical regression modules beneath `tests/integration/mcp/`: protocol, tools, read roots, configuration, snapshots, issues, Gradle, workspaces, and limits. Cross-topic fixtures remain at the facade; topic-specific helpers stay with their callers. Every existing test declaration, complete body, literal, assertion, platform attribute, and debug-only rendezvous ignore guard is preserved. The root integration harness registration remains unchanged, and compiled inventory comparisons verify no regression is omitted. This internal organization changes no MCP behavior or production API.
