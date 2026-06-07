---
spec: commands.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/mod.rs` | cargo test commands::mod | No inline tests found; add focused coverage for `load_and_discover`, `filter_specs`, `filter_by_status`, `build_schema_columns` before risky changes |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | End-to-end fixture: `strict_turns_warnings_into_errors` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration root_flag_overrides_cwd | End-to-end fixture: `root_flag_overrides_cwd` |
| `tests/integration.rs` | cargo test --test integration default_command_is_check | End-to-end fixture: `default_command_is_check` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration strict_on_coverage_subcommand | End-to-end fixture: `strict_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration action_validates_require_coverage_input | End-to-end fixture: `action_validates_require_coverage_input` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Filter by module name | specs exist at `specs/auth/auth.spec.md` and `specs/api/api.spec.md` | `filter_specs(root, specs, &["auth"])` is called | returns only `specs/auth/auth.spec.md` |
| Strict mode with warnings | enforcement is `Strict`, `--strict` is set, validation has 0 errors but 3 warnings | `compute_exit_code()` is called | returns 1 (warnings treated as errors) |
| EnforceNew with unspecced files | enforcement is `EnforceNew`, coverage shows 2 unspecced files | `exit_with_status()` is called | prints count and exits with code 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No spec files found and `allow_empty` is false | Prints suggestion to run `specsync generate` and exits 0 | Keep or add a focused assertion before changing this behavior |
| Filter matches no specs | Prints warning listing unmatched filters, returns empty vec | Keep or add a focused assertion before changing this behavior |
| `schema_dir` not configured | `build_schema_columns` returns empty map (no error) | Keep or add a focused assertion before changing this behavior |
| GitHub repo unresolvable for drift issues | Prints error and returns without creating issues | Keep or add a focused assertion before changing this behavior |
| `gh` CLI fails to create issue | Prints per-spec error but continues with remaining specs | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/mod.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
