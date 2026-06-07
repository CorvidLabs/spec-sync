---
spec: cmd_scaffold.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/scaffold.rs` | cargo test commands::scaffold | No inline tests found; add focused coverage for `cmd_add_spec`, `cmd_scaffold`, `load_config`, `get_exported_symbols` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Scaffold with auto-detection" before changing user-visible CLI output, generated files, or error handling in cmd_scaffold.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Scaffold with auto-detection | `src/auth.rs` exists | `cmd_add_spec(root, "auth")` runs | creates spec with detected sources and companions (including design.md if `companions.design` is enabled) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec exists | Early return | Keep or add a focused assertion before changing this behavior |
| Dir creation fails | Exits 1 | Keep or add a focused assertion before changing this behavior |
| Custom template dir missing | Falls back to built-in | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- scaffold --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/scaffold.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
