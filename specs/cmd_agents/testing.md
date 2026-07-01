---
spec: cmd_agents.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/agents.rs` | (none) | No inline `#[cfg(test)]` module and no integration fixtures target this dispatcher. Coverage is indirect via the `agents` module. |
| `src/agents.rs` | cargo test agents | The `agents` library module owns install/uninstall/status tests; verify target-to-artifact mapping there. |

## Manual Testing

- [x] `specsync agents install --claude --gemini` on a clean project installs only Claude's and Gemini's artifacts
- [x] `specsync agents install` (no flags) installs all four tools

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No target flags on install/uninstall | Empty target vec is passed and treated as "all tools" by `agents`, matching `cmd_hooks`'s convention |
| Skill/command write fails inside `agents` | Error reporting is delegated to the `agents` module; this dispatcher does not catch or wrap errors |
