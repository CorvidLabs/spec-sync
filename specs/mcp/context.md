---
spec: mcp.spec.md
---

## Key Decisions

- MCP is a deterministic stdio JSON-RPC adapter for coding agents.
- Generate creates local templates only and rejects retired inference arguments.
- Tool errors use `isError`; protocol errors remain JSON-RPC errors.
- Agent credentials and model execution stay outside SpecSync.

## Files to Read First

- `src/mcp.rs`
- `src/generator.rs`
- `src/validator.rs`

## Current Status

Stable agent-native MCP integration without embedded provider or credential surfaces.
