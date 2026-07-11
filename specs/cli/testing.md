---
spec: cli.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/cli.rs` (clap grammar) | cargo test cli::tests | `no_subcommand_yields_none_and_text_default`, `global_flags_parse_before_subcommand`, `json_format_value_enum_parses`, `check_collects_flags_and_positional_specs`, `stale_threshold_defaults_and_overrides`, `exclude_status_splits_on_commas`, `unknown_subcommand_is_rejected`, `non_numeric_threshold_is_rejected` |
| `src/main.rs` (exit codes) | cargo test --bin specsync tests:: | `warn_mode_exits_0_with_no_errors`, `warn_mode_exits_0_even_with_errors`, `warn_mode_exits_0_even_with_strict_flag`, `warn_mode_respects_require_coverage`, `enforce_new_exits_0_when_all_files_specced`, `enforce_new_exits_1_when_unspecced_files_exist`, `strict_mode_exits_1_with_warnings_and_strict_flag`, `strict_mode_respects_require_coverage` |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | End-to-end fixture: `strict_turns_warnings_into_errors` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration root_flag_overrides_cwd | End-to-end fixture: `root_flag_overrides_cwd` |
| `tests/integration.rs` | cargo test --test integration default_command_is_check | End-to-end fixture: `default_command_is_check` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration strict_on_coverage_subcommand | End-to-end fixture: `strict_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration toml_config_is_loaded | End-to-end fixture: `toml_config_is_loaded` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Default subcommand | the user runs `specsync` with no subcommand | the CLI parses arguments | the `check` command executes |
| Strict mode with warnings | specs have undocumented exports (warnings but no errors) | `specsync check --strict` is run | the process exits with code 1 |
| JSON output | `--json` flag is passed | any command runs | output is valid JSON with no ANSI escape codes |
| Init idempotency | `specsync.json` already exists in the project root | `specsync init` is run | prints "specsync.json already exists" and returns without modifying it |
| Coverage threshold | file coverage is 80% | `specsync check --require-coverage 90` is run | the process exits with code 1 and prints the unspecced files |
| Deterministic generate | uncovered modules exist | `specsync generate` is run | local template specs and companions are created |
| Retired provider/model flags | user supplies `--provider` or `--model` | Clap parses arguments | unknown arguments are rejected |
| Panic is caught | a subcommand panics internally | the binary runs | a "specsync panicked … please report it" message is printed and the process exits 1 (no raw backtrace) |
| Resolve without network | specs have cross-project `depends_on` refs | `specsync resolve` is run (without `--remote`) | lists the refs but does not verify them against remote registries |
| Fix auto-adds undocumented exports | a spec's source files have exports not documented in the Public API section | `specsync check --fix` is run | skeleton rows for the missing exports are appended to the Public API section and the spec file is written to disk |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Cannot determine cwd | Panics with "Cannot determine cwd" | Keep or add a focused assertion before changing this behavior |
| Failed to write `specsync.json` | Panics with "Failed to write specsync.json" | Keep or add a focused assertion before changing this behavior |
| Failed to create spec directory | Prints error to stderr and exits 1 | Keep or add a focused assertion before changing this behavior |
| Failed to write spec file | Prints error to stderr and exits 1 | Keep or add a focused assertion before changing this behavior |
| Failed to write `specsync-registry.toml` | Prints error to stderr and exits 1 | Keep or add a focused assertion before changing this behavior |
| No spec files found (non-generate commands) | Prints guidance message and exits 0 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/main.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
