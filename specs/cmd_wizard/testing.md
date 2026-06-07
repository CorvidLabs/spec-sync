---
spec: cmd_wizard.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/wizard.rs` | cargo test commands::wizard | No inline tests found; add focused coverage for `cmd_wizard`, `load_config` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Create API endpoint spec" before changing user-visible CLI output, generated files, or error handling in cmd_wizard.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Create API endpoint spec | user selects "API Endpoint" template | wizard generates spec content | includes Endpoints table section with Method, Path, Description columns |
| Auto-detect source files | module name "auth", `src/auth.rs` exists | wizard runs source detection | pre-fills source files with `src/auth.rs` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Empty module name entered | Exits with code 1 | Keep or add a focused assertion before changing this behavior |
| Spec directory already exists | Prints error and exits 1 | Keep or add a focused assertion before changing this behavior |
| User cancels at confirmation | Exits cleanly with code 0 | Keep or add a focused assertion before changing this behavior |
| Directory creation fails | Exits with code 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- wizard --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/wizard.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
