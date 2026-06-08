---
spec: cmd_coverage.spec.md
---

## Key Decisions

- Validation runs first (`run_validation`), then `compute_coverage`, so the coverage report and any enforcement gate stay consistent with `check`.
- Two output paths: JSON (`--format json`) builds a `serde_json` object and always `exit(0)` — it is a metrics dump, not a gate. The non-JSON path prints the report + summary + coverage line and delegates the gate to `exit_with_status`.
- Percentages are computed inline in the wrapper (file_coverage, loc_coverage), rounded to two decimals, with a zero denominator treated as 100%.
- This command uses `IgnoreRules::default()` rather than `IgnoreRules::load(root)` — `.specsyncignore` is intentionally not applied to coverage discovery here.

## Files to Read First

- `src/commands/coverage.rs` — the command wrapper (this module), including the JSON serialization
- `src/validator.rs` — `compute_coverage` (the `Coverage` struct fields used in the JSON) and `get_schema_table_names`
- `src/commands/mod.rs` — `run_validation`, `exit_with_status`, `load_and_discover`, `build_schema_columns`
- `src/output.rs` — `print_coverage_report`, `print_summary`, `print_coverage_line`

## Current Status

Implemented and stable. Well covered by `tests/integration.rs` (8 fixtures spanning full/partial coverage, `--require-coverage`, `--strict`, and the MCP coverage tool). The wrapper has no inline unit tests.

## Notes

- JSON keys: `file_coverage`, `files_covered`, `files_total`, `loc_coverage`, `loc_covered`, `loc_total`, `modules`, `uncovered_files`.
