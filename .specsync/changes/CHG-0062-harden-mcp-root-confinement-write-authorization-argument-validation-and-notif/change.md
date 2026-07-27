---
id: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
state: accepted
type: feature
base_commit: ce669736042b46a04b14cf8f86312ba75cb52c33
---

# Harden MCP root confinement, write authorization, argument validation, and notification semantics for issue 414

## Intent

Harden MCP root confinement, write authorization, argument validation, and notification semantics for issue 414

## Affected Canonical Specs

- `mcp`
- `cli_args`
- `cli`

## Acceptance Criteria

- MCP is read-only by default; mutating tools require --allow-write, are hidden from tools/list in read-only mode, and direct calls are rejected before execution; mutators cannot override the server root; read roots must be canonical descendants; traversal and symlink escapes are rejected; tool arguments are exact-schema validated with JSON-RPC -32602 for protocol-shape errors; every notification including unknown methods produces no response and cannot mutate; regression tests prove paths outside the root remain unchanged.

## No-spec Rationale

Not applicable
