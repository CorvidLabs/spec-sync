---
spec: cmd_wizard.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/wizard.rs` | cargo test commands::wizard | No inline `#[cfg(test)]` module; the flow is interactive (TTY-bound via `dialoguer`) so it is not exercised by automated tests |

## Coverage Gaps

- The interactive wizard cannot be driven by the standard CLI integration harness (it blocks on TTY prompts). The spec-body template and companion generation it relies on are covered indirectly by the `generate`/scaffold companion tests (`generate_creates_companion_files`, `generate_creates_design_md_when_enabled`). Verify the template-specific sections manually before changing them.

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
