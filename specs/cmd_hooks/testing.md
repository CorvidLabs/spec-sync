---
spec: cmd_hooks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/hooks.rs` | cargo test commands::hooks | No inline tests found; add focused coverage for `cmd_hooks` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Install specific hooks" before changing user-visible CLI output, generated files, or error handling in cmd_hooks.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Install specific hooks | `specsync hooks install --claude --precommit` | `cmd_hooks` runs | installs only CLAUDE.md and pre-commit hook |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Hook write fails | Delegated to hooks module | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- hooks --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/hooks.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
