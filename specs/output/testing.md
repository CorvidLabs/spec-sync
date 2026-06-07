---
spec: output.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/output.rs` | cargo test output | No inline tests found; add focused coverage for `print_summary`, `print_coverage_line`, `print_coverage_report`, `print_check_markdown` before risky changes |
| `tests/integration.rs` | cargo test --test integration score_command_outputs_quality_grades | End-to-end fixture: `score_command_outputs_quality_grades` |
| `tests/integration.rs` | cargo test --test integration score_json_output_has_grades | End-to-end fixture: `score_json_output_has_grades` |
| `tests/integration.rs` | cargo test --test integration fix_with_json_output | End-to-end fixture: `fix_with_json_output` |
| `tests/integration.rs` | cargo test --test integration diff_human_readable_output | End-to-end fixture: `diff_human_readable_output` |
| `tests/integration.rs` | cargo test --test integration score_all_format_table_outputs_headers | End-to-end fixture: `score_all_format_table_outputs_headers` |
| `tests/integration.rs` | cargo test --test integration score_all_format_csv_outputs_header_row | End-to-end fixture: `score_all_format_csv_outputs_header_row` |
| `tests/integration.rs` | cargo test --test integration migrate_json_output_format | End-to-end fixture: `migrate_json_output_format` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| All specs pass | 25 specs checked, 25 passed, 0 warnings | `print_summary(25, 25, 0, 0)` is called | output: `25 specs checked: 25 passed, 0 warning(s), 0 failed` (25 in green, 0 in green) |
| Coverage below 80% | coverage is 58% file coverage | `print_coverage_line()` is called | percentage is displayed in red |
| Diff with no changes | no spec-tracked source files changed since base ref | `print_diff_markdown()` is called with empty entries | prints "No spec-tracked source files changed since `{base}`." |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Empty spec list | `print_summary` shows "0 passed, 0 failed" | Keep or add a focused assertion before changing this behavior |
| Coverage report with no unspecced files | Shows "✓ All source files referenced by specs" | Keep or add a focused assertion before changing this behavior |
| Diff with changed files not in any spec | Lists them under "Changed files not covered by any spec" | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/output.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
