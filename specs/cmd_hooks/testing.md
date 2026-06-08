---
spec: cmd_hooks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/hooks.rs` | (none) | No inline `#[cfg(test)]` module and no integration fixtures target this dispatcher. Coverage is indirect via the `hooks` module. |
| `src/hooks.rs` | cargo test hooks | The `hooks` library module owns install/uninstall/status tests; verify target-to-file mapping there. |

## Coverage Gaps

- No fixture exercises the flag-to-`HookTarget` mapping in `collect_hook_targets`. Add a `hooks install --claude --precommit` integration test that asserts only `CLAUDE.md` and the pre-commit hook are written, and that other targets are absent, before changing the mapping.
- No fixture asserts the "no flags → all targets" empty-vec convention.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Install specific targets | empty temp project | `specsync hooks install --claude --precommit` | Only `CLAUDE.md` and the git pre-commit hook are installed. |
| Install all (no flags) | empty temp project | `specsync hooks install` | All targets installed (empty target vec → all). |
| Status | project with some hooks installed | `specsync hooks status` | Reports installed/not-installed for each target. |
| Uninstall specific targets | project with hooks installed | `specsync hooks uninstall --claude` | Removes only the CLAUDE.md instructions. |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No target flags on install/uninstall | Empty target vec is passed and treated as "all targets" by `hooks` | Keep; assert via a fixture before changing the empty-vec convention. |
| Hook write fails | Error reporting is delegated to the `hooks` module | Cover in `hooks` module tests, not here. |

## Reviewer Checklist

- Run `cargo run -- hooks --help` and confirm the documented flags (`--claude`, `--cursor`, `--copilot`, `--agents`, `--precommit`, `--claude-code-hook`) are present.
- When changing `collect_hook_targets`, add or update a fixture that asserts the resulting `HookTarget` set.
- For changes to generated file content or write behavior, run the `hooks` module's tests — that is where the I/O lives.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
