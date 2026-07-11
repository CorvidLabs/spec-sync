---
spec: cli_args.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/cli.rs` (inline `#[cfg(test)] mod tests`) | cargo test --bin specsync cli:: | 8 parser tests: `no_subcommand_yields_none_and_text_default`, `global_flags_parse_before_subcommand`, `json_format_value_enum_parses`, `check_collects_flags_and_positional_specs`, `stale_threshold_defaults_and_overrides`, `exclude_status_splits_on_commas`, `unknown_subcommand_is_rejected`, `non_numeric_threshold_is_rejected` |
| Verified SDD grammar | cargo test --bin specsync cli::tests::change_new_collects_sdd_scope | `ChangeAction::New` collects kind, repeatable specs/paths/artifacts, and no-spec rationale flags |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | Global `--strict` turns warnings into errors |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | `--require-coverage` exits non-zero below threshold |
| `tests/integration.rs` | cargo test --test integration root_flag_overrides_cwd | `--root` overrides the working directory |
| `tests/integration.rs` | cargo test --test integration provider_flag_unknown_provider_errors | Unknown `--provider` value errors with "Unknown provider" |
| `tests/integration.rs` | cargo test --test integration provider_flag_enables_ai | `--provider` switches `generate` into AI mode |
| `tests/integration.rs` | cargo test --test integration cli_provider_overrides_config_provider | `--provider` outranks the `aiProvider` config value |
| `tests/integration.rs` | cargo test --test integration env_provider_overrides_config_provider | `SPECSYNC_AI_PROVIDER` env outranks `aiProvider` config (flag > env > config) |
| `tests/integration.rs` | cargo test --test integration unknown_provider_lists_api_options | Unknown-provider error lists the API options |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Global strict flag propagates to subcommand | user runs `specsync check --strict` | Clap parses arguments | `Cli.strict == true` is accessible regardless of the `Check` subcommand |
| Default subcommand | user runs `specsync` with no subcommand | Clap parses arguments | `Cli.command` is `None`, and `main.rs` defaults to Check behavior |
| Hooks install targets | user runs `specsync hooks install --claude --precommit` | Clap parses arguments | `HooksAction::Install { claude: true, precommit: true, ... }` with all others false |
| Generate provider + model flags | user runs `specsync generate --provider anthropic --model claude-x` | Clap parses arguments | `Command::Generate { provider: Some("anthropic"), model: Some("claude-x"), .. }` |
| Auto-detect sentinel | user runs `specsync generate --provider auto` | Clap parses arguments | `provider == Some("auto")`; `generate.rs` treats this as force-auto-detect |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Unknown subcommand | Clap prints error with usage help and exits non-zero | Keep or add a focused assertion before changing this behavior |
| Missing required argument (e.g., `new` without name) | Clap prints error listing required args | Keep or add a focused assertion before changing this behavior |
| Invalid `--enforcement` value | Clap prints accepted values: warn, enforce-new, strict | Keep or add a focused assertion before changing this behavior |
| Invalid `--format` value | Clap prints accepted values: text, json, markdown, github, table, csv | Keep or add a focused assertion before changing this behavior |
| Unknown `--provider` value | Deferred to `resolve_ai_provider`, which errors and lists available providers (`unknown_provider_lists_api_options`) | Parser stays loose; do not turn `--provider` into a typed enum |
| Non-numeric `--threshold` / `--require-coverage` | Clap rejects with a parse error | `non_numeric_threshold_is_rejected` pins this |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/cli.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
