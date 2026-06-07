---
spec: types.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/types.rs` inline tests | Unit | Validate Types behavior close to implementation, especially `AiProvider`, `Language`, `OutputFormat`, `ExportLevel`, `SpecStatus`, `EnforcementMode`, `CustomRuleType`, `RuleSeverity` |
| `tests/integration.rs` | Integration | Exercise Types through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Types contracts or source files.
- [ ] Run `fledge run test` and confirm Types unit/integration coverage still passes.
- [ ] Review examples in `types.spec.md` against observed behavior when touching src/types.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Unknown provider string | `AiProvider::from_str_loose` returns `None` |
| Unsupported file extension | `Language::from_extension` returns `None` |
| Invalid JSON config | `SpecSyncConfig` deserialization fails at the caller level |
