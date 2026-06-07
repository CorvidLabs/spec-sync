---
spec: cmd_new.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/new.rs` | cargo test commands::new | No inline tests found; add focused coverage for `cmd_new`, `load_config`, `generate_companion_files` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Quick spec" before changing user-visible CLI output, generated files, or error handling in cmd_new.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Quick spec | `src/auth.rs` exists | `cmd_new(root, "auth", false)` runs | creates `specs/auth/auth.spec.md` with detected source and exports |
| Full with companions | `--full` flag | `cmd_new` runs | creates spec.md, tasks.md, context.md, requirements.md, testing.md (and design.md if `companions.design` is enabled) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec already exists | Exits 1 | Keep or add a focused assertion before changing this behavior |
| No source files found | Creates spec with empty `files:` | Keep or add a focused assertion before changing this behavior |
| Dir creation fails | Exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- new --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/new.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
