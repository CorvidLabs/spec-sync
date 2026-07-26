---
spec: rehash.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/rehash.rs` | `cargo test commands::rehash` | `discover_spec_files_honors_config_and_excludes_templates`, `cmd_rehash_rebuilds_a_fresh_cache` |
| `tests/integration.rs` | `cargo test --test integration rehash_writes_complete_warm_validation_snapshots` | Config hash, format version, complete snapshot, warm counters, and preserved warning |
| `tests/integration.rs` | `cargo test --test integration rehash_does_not_publish_snapshots_when_validation_has_errors` | Error-bearing rebuild clears snapshots and forces the next check to validate |

## Coverage Gaps

- Integration gap: add a fixture for "Normal rehash" before changing user-visible CLI output, generated files, or error handling in rehash.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Normal rehash | a valid specsync project with specs | `cmd_rehash(root)` runs | writes fresh hashes.json and prints spec count |
| Warm follow-up | warning-only project was rehashed and no input changed | run JSON check | reports the warning with full checked count and cached count, without fresh spec validation |
| Save failure | .specsync directory is not writable | `cmd_rehash(root)` runs | prints error and exits with code 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cache save fails | Prints error, exits 1 | Keep or add a focused assertion before changing this behavior |
| Rehash output contains only hashes | Must also contain a current compatible snapshot for every discovered spec | `rehash_writes_complete_warm_validation_snapshots` |
| Rehash validation has errors | Clears replayable snapshots; next check validates and reports errors | `rehash_does_not_publish_snapshots_when_validation_has_errors` |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/commands/rehash.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
