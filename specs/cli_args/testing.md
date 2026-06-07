---
spec: cli_args.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/cli.rs` | cargo test cli | No inline tests found; add focused coverage for `Cli`, `Command`, `HooksAction`, `LifecycleAction` before risky changes |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | End-to-end fixture: `strict_turns_warnings_into_errors` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration root_flag_overrides_cwd | End-to-end fixture: `root_flag_overrides_cwd` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration strict_on_coverage_subcommand | End-to-end fixture: `strict_on_coverage_subcommand` |
| `tests/integration.rs` | cargo test --test integration provider_flag_unknown_provider_errors | End-to-end fixture: `provider_flag_unknown_provider_errors` |
| `tests/integration.rs` | cargo test --test integration provider_flag_enables_ai | End-to-end fixture: `provider_flag_enables_ai` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Global strict flag propagates to subcommand | user runs `specsync check --strict` | Clap parses arguments | `Cli.strict == true` is accessible regardless of the `Check` subcommand |
| Default subcommand | user runs `specsync` with no subcommand | Clap parses arguments | `Cli.command` is `None`, and `main.rs` defaults to Check behavior |
| Hooks install targets | user runs `specsync hooks install --claude --precommit` | Clap parses arguments | `HooksAction::Install { claude: true, precommit: true, ... }` with all others false |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Unknown subcommand | Clap prints error with usage help and exits non-zero | Keep or add a focused assertion before changing this behavior |
| Missing required argument (e.g., `new` without name) | Clap prints error listing required args | Keep or add a focused assertion before changing this behavior |
| Invalid `--enforcement` value | Clap prints accepted values: warn, enforce-new, strict | Keep or add a focused assertion before changing this behavior |
| Invalid `--format` value | Clap prints accepted values: text, json, markdown | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/cli.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
