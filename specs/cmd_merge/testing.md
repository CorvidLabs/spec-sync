---
spec: cmd_merge.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/merge.rs` | cargo test commands::merge | No inline tests found; add focused coverage for `cmd_merge`, `load_config`, `OutputFormat` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Auto-resolved" before changing user-visible CLI output, generated files, or error handling in cmd_merge.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Auto-resolved | 3 specs with simple conflicts | `cmd_merge` runs | all auto-resolved |
| Manual needed | complex conflict | `cmd_merge` runs | flags file, exits 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No conflicts | Prints "No spec files with merge conflicts found." | Keep or add a focused assertion before changing this behavior |
| Complex conflict | Prints "needs manual merge: <path>" and exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- merge --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/merge.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
