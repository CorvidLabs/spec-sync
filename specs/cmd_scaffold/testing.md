---
spec: cmd_scaffold.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/scaffold.rs` | cargo test commands::scaffold | Inline tests cover `validate_module_name` (accepts plain names, rejects separators/`..`/absolute); still add coverage for source auto-detection and registry auto-registration before risky changes |
| `tests/integration.rs` | cargo test --test integration scaffold_rejects_module_name_path_traversal | `add-spec`/`scaffold` with a traversal name (`../../escape/evil`) exit non-zero with "invalid module name" and write nothing outside the project root |
| `tests/integration.rs` | cargo test --test integration generate_creates_companion_files | Exercises the same `generator::generate_companion_files_for_spec` path scaffold uses to emit companions |
| `tests/integration.rs` | cargo test --test integration companion_files_not_overwritten_on_regenerate | Confirms existing companions are not clobbered when re-running on an existing spec |

## Coverage Gaps

- Integration gap: add a fixture for "Scaffold with auto-detection" before changing user-visible CLI output, generated files, or error handling in cmd_scaffold.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Scaffold with auto-detection | `src/auth.rs` exists | `cmd_add_spec(root, "auth")` runs | creates spec with detected sources and companions (including design.md if `companions.design` is enabled) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec exists | Early return | Keep or add a focused assertion before changing this behavior |
| Dir creation fails | Exits 1 | Keep or add a focused assertion before changing this behavior |
| Custom template dir missing | Falls back to built-in | Keep or add a focused assertion before changing this behavior |
| Module name with path separator / `..` / absolute / empty | Refused before any write; exits 1 with "invalid module name" (no path traversal) | Asserted by `scaffold_rejects_module_name_path_traversal` + `commands::scaffold::tests`; keep the guard first in both entry points |

## Reviewer Checklist

- Run `cargo run -- scaffold --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/scaffold.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
