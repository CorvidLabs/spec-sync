---
spec: ai.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/ai.rs` inline tests | Unit | Validate Ai behavior close to implementation, especially `ResolvedProvider`, `resolve_ai_provider`, `resolve_ai_command`, `generate_spec_with_ai`, `regenerate_spec_with_ai` |
| `tests/integration.rs` | Integration | Exercise Ai through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Ai contracts or source files.
- [ ] Run `fledge run test` and confirm Ai unit/integration coverage still passes.
- [ ] Review examples in `ai.spec.md` against observed behavior when touching src/ai.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No AI provider found | Returns descriptive error listing all options |
| Provider binary not installed | Error: "not installed or not on PATH" |
| API key missing | Error: "requires an API key. Set ENV_VAR or add aiApiKey" |
| Cursor selected as provider | Error explaining no CLI pipe mode, with workarounds |
