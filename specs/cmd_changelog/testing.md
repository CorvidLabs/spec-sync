---
spec: cmd_changelog.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/changelog.rs` | cargo test commands::changelog | No inline tests found; add focused coverage for `cmd_changelog`, `generate_changelog`, `load_config`, `OutputFormat` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Valid range with changes" before changing user-visible CLI output, generated files, or error handling in cmd_changelog.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Valid range with changes | specs changed between `v3.5.0` and `v3.6.0` | `cmd_changelog(root, "v3.5.0..v3.6.0", Text)` is called | prints list of added, modified, and removed specs |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Range missing `..` | Prints error and exits 1 | Keep or add a focused assertion before changing this behavior |
| Invalid git refs | Git command fails, error propagated | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- changelog --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/changelog.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
