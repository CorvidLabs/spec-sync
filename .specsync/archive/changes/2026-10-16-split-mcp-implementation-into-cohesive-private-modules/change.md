---
id: split-mcp-implementation-into-cohesive-private-modules
state: archived
type: refactor
base_commit: 621a74785e3c09b2e9ebea35768e05f5772f886f
---

# Split MCP implementation into cohesive private modules

## Intent

Split MCP implementation into cohesive private modules

## Affected Canonical Specs

- `mcp`

## Acceptance Criteria

- Replace textual MCP includes with ordinary private Rust modules while preserving existing production function bodies and cfg gates; keep run_mcp_server and mcp_tool_definitions as the two external MCP entry points; conserve all existing MCP tests and scan every moved source for provider-process construction; preserve protocol responses, confinement, budgets, publication, rollback and debug race barriers; pass debug unit/integration inventories, release-test compilation and strict canonical contract checks.

## No-spec Rationale

Not applicable
