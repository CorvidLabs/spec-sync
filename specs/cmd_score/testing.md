---
spec: cmd_score.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/score.rs` | cargo test commands::score | No inline tests found; add focused coverage for `cmd_score`, `OutputFormat` before risky changes |
| `tests/integration.rs` | cargo test --test integration score_command_outputs_quality_grades | End-to-end fixture: `score_command_outputs_quality_grades` |
| `tests/integration.rs` | cargo test --test integration score_json_output_has_grades | End-to-end fixture: `score_json_output_has_grades` |
| `tests/integration.rs` | cargo test --test integration mcp_score_tool_returns_grades | End-to-end fixture: `mcp_score_tool_returns_grades` |
| `tests/integration.rs` | cargo test --test integration score_all_format_table_outputs_headers | End-to-end fixture: `score_all_format_table_outputs_headers` |
| `tests/integration.rs` | cargo test --test integration score_all_format_csv_outputs_header_row | End-to-end fixture: `score_all_format_csv_outputs_header_row` |
| `tests/integration.rs` | cargo test --test integration score_all_format_csv_includes_summary_row | End-to-end fixture: `score_all_format_csv_includes_summary_row` |
| `tests/integration.rs` | cargo test --test integration score_format_table_without_all_flag_still_works | End-to-end fixture: `score_format_table_without_all_flag_still_works` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Score with explain | `--explain` set | `cmd_score` runs | each spec shows FM/Sec/API/Depth/Fresh subscores |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs match filters | Warning printed | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- score --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/score.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
