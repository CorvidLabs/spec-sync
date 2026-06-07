---
spec: cmd_init_registry.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/init_registry.rs` | cargo test commands::init_registry | No inline tests found; add focused coverage for `cmd_init_registry`, `load_config`, `generate_registry` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "Generate registry" before changing user-visible CLI output, generated files, or error handling in cmd_init_registry.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Generate registry | 25 specs, no existing registry | `cmd_init_registry(root, None)` runs | creates TOML with 25 entries |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Registry exists | Early return | Keep or add a focused assertion before changing this behavior |
| Write fails | Exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- init-registry --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/init_registry.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
