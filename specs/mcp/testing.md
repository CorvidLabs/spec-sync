---
spec: mcp.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/mcp.rs` inline tests | Unit | Validate Mcp behavior close to implementation, especially `run_mcp_server`, `()`, `generate_specs_for_unspecced_modules_paths`, `resolve_ai_provider`, `parse_frontmatter`, `SpecSyncConfig` |
| `tests/integration.rs` | Integration | Exercise Mcp through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Mcp contracts or source files.
- [ ] Run `fledge run test` and confirm Mcp unit/integration coverage still passes.
- [ ] Review examples in `mcp.spec.md` against observed behavior when touching src/mcp.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Malformed JSON input | JSON-RPC error -32700 "Parse error" |
| Unknown method with id | JSON-RPC error -32601 "Method not found" |
| Unknown tool name | Tool error: "Unknown tool: {name}" |
| Unknown resource URI | JSON-RPC error -32602 "Unknown resource URI: {uri}" |
