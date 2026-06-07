---
spec: cmd_rules.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/rules.rs` | cargo test commands::rules | No inline tests found; add focused coverage for `cmd_rules`, `print_builtin`, `load_config` before risky changes |
| `tests/integration.rs` | cargo test --test integration write_config_with_custom_rules | End-to-end fixture: `write_config_with_custom_rules` |
| `tests/integration.rs` | cargo test --test integration rules_command_lists_custom_rules | End-to-end fixture: `rules_command_lists_custom_rules` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| No custom rules defined | `specsync.json` has no `customRules` array | `specsync rules` runs | built-in rules are listed, followed by "No custom rules defined." with guidance text |
| Custom rules with filters | a custom rule with `appliesTo: { status: "stable", module: "^auth" }` | `specsync rules` runs | the rule shows `applies_to: status=stable, module=/^auth/` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Missing `specsync.json` | Config loader handles this (not this module's concern) | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- rules --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/rules.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
