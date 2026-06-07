---
spec: cmd_view.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/view.rs` | cargo test commands::view | No inline tests found; add focused coverage for `cmd_view`, `load_config`, `find_spec_files`, `view_spec` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Dev view" before changing user-visible CLI output, generated files, or error handling in cmd_view.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Dev view | `specsync view --role dev --spec auth` | `cmd_view` runs | renders auth spec with dev-relevant sections only |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs found | Exits 1 | Keep or add a focused assertion before changing this behavior |
| Spec read error | Error printed, continues | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- view --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/view.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
