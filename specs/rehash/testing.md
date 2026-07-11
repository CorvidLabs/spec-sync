---
spec: rehash.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/rehash.rs` | `cargo test commands::rehash` | `discover_spec_files_honors_config_and_excludes_templates`, `cmd_rehash_rebuilds_a_fresh_cache` |

## Coverage Gaps

- Integration gap: add a fixture for "Normal rehash" before changing user-visible CLI output, generated files, or error handling in rehash.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Normal rehash | a valid specsync project with specs | `cmd_rehash(root)` runs | writes fresh hashes.json and prints spec count |
| Save failure | .specsync directory is not writable | `cmd_rehash(root)` runs | prints error and exits with code 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cache save fails | Prints error, exits 1 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/commands/rehash.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
