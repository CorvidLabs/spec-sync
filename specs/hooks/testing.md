---
spec: hooks.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/hooks.rs` inline tests | Unit | Validate Hooks behavior close to implementation, especially `HookTarget`, `all`, `name`, `description`, `from_str`, `Option<Self>`, `is_installed`, `bool` |
| `tests/integration.rs` | Integration | Exercise Hooks through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Hooks contracts or source files.
- [ ] Run `fledge run test` and confirm Hooks unit/integration coverage still passes.
- [ ] Review examples in `hooks.spec.md` against observed behavior when touching src/hooks.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Cannot read/write file | Returns `Err` with descriptive message |
| Cannot create directory | Returns `Err` with descriptive message |
| Uninstall Claude Code hook | Returns `Err` — must be removed manually |
| Cannot parse existing settings.json | Returns `Err` with parse error |
