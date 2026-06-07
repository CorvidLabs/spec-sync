---
spec: cmd_check.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/check.rs` | cargo test commands::check | No inline tests found; add focused coverage for `cmd_check`, `IgnoreRules::load`, `build_comment_body`, `resolve_repo` before risky changes |
| `tests/integration.rs` | cargo test --test integration check_valid_project_passes | End-to-end fixture: `check_valid_project_passes` |
| `tests/integration.rs` | cargo test --test integration check_missing_source_file_fails | End-to-end fixture: `check_missing_source_file_fails` |
| `tests/integration.rs` | cargo test --test integration check_undocumented_export_warns | End-to-end fixture: `check_undocumented_export_warns` |
| `tests/integration.rs` | cargo test --test integration check_phantom_export_errors | End-to-end fixture: `check_phantom_export_errors` |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | End-to-end fixture: `strict_turns_warnings_into_errors` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Incremental check with cache | 25 specs, 3 have changed since last check | `cmd_check` runs without `--force` | only 3 specs are validated; 22 are skipped via hash cache |
| Auto-fix undocumented exports | spec is missing export `pub fn new_function()` | `cmd_check` runs with `--fix` | the export is appended to the spec's Public API table with a generated description prompt and the file is rewritten |
| JSON output format | `--format json` is set | validation completes with errors and warnings | output is a single JSON object with `specs_checked`, `passed`, `errors`, `warnings`, `coverage`, and `exit_code` fields |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| AI provider not available during `--fix` regen | Prints error per spec, continues with remaining specs | Keep or add a focused assertion before changing this behavior |
| Auto-fix changes a spec but validation still fails | Reports remaining errors, does not loop | Keep or add a focused assertion before changing this behavior |
| Hash cache file is corrupted | Falls back to full validation (cache miss) | Keep or add a focused assertion before changing this behavior |
| `--create-issues` with no GitHub repo | Prints error, skips issue creation | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- check --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/check.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
