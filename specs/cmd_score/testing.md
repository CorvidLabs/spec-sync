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
| Minimum gate | `cargo test --test integration score_minimum_gate_and_strict_reject_untouched_scaffolds` | Advisory default, explicit 80 gate, strict-implied 80, and parseable JSON failure |
| Threshold validation | `cargo test --test integration score_rejects_minimum_above_one_hundred_as_usage_error` | Values above 100 exit 2 |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Score with explain | `--explain` set | `cmd_score` runs | each spec shows FM/Sec/API/Depth/Fresh subscores |
| CSV format | `--format csv --all` | `specsync score` runs | a header row, one row per spec, and a final `SUMMARY` row are printed (`score_all_format_csv_outputs_header_row`, `score_all_format_csv_includes_summary_row`) |
| Table format | `--format table --all` | `specsync score` runs | an aligned ASCII table with a Spec/Score/Grade header is printed (`score_all_format_table_outputs_headers`) |
| JSON grades | `--format json` (or `--json`) | `specsync score` runs | JSON includes per-spec `grade`/`total` and a project `average_score`/`distribution` (`score_json_output_has_grades`) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs match filters | Warning printed (via `filter_specs`) | Keep or add a focused assertion before changing this behavior |
| `--format table` without `--all` | Still renders a valid table (single/filtered spec) | Covered by `score_format_table_without_all_flag_still_works` |
| No quality gate | `score` remains advisory and exits 0 | Protected alongside the minimum-gate fixture |
| `--min-score` / `--strict` | Any selected score below the effective minimum exits 1 | Keep text and JSON regressions |

## Reviewer Checklist

- Run `cargo run -- score --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/score.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
