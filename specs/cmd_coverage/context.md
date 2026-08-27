---
spec: cmd_coverage.spec.md
---

## Key Decisions

- Validation runs first (`run_validation`), then `compute_coverage_checked`, so the coverage report and any enforcement gate stay consistent with `check`.
- Both output paths honor gates. Malformed manifest discovery exits 1; JSON remains parseable with `valid: false`, `inconclusive: true`, null percentages, empty result collections, and an explicit error.
- Percentages are serialized by `output::percent_json`, rounded to two decimals. A zero denominator yields `null`, never a fabricated 100% (#582) — a badge or dashboard cannot tell a made-up 100 from a real one.
- This command uses `IgnoreRules::default()` rather than `IgnoreRules::load(root)` — `.specsyncignore` is intentionally not applied to coverage discovery here.

## Files to Read First

- `src/commands/coverage.rs` — the command wrapper (this module), including the inconclusive-discovery JSON payload
- `src/output.rs` — `coverage_json` and `percent_json`, which build the success-path JSON document
- `src/validator.rs` — `compute_coverage_checked` and `get_schema_table_names`; the fields feeding the JSON live on `types::CoverageReport`, not on any struct named `Coverage`
- `src/commands/mod.rs` — `run_validation`, `exit_with_status`, `load_and_discover`, `build_schema_columns`
- `src/output.rs` — `print_coverage_report`, `print_summary`, `print_coverage_line`

## Current Status

Implemented and stable. Well covered under `tests/integration/` — 34 test functions invoke the `coverage` subcommand, spread across `check.rs`, `commands.rs`, `coverage_phantom_module.rs`, `coverage_unmeasured.rs`, `finding_identity_parity.rs`, and `regression_w1.rs`, spanning full/partial coverage, `--require-coverage`, `--strict`, unmeasurable denominators, and the MCP coverage tool. The wrapper has no inline unit tests.

## Notes

- JSON keys, all 21 of them, built by `output::coverage_json`: gate verdict (`passed`, `specs_checked`, `specs_passed`), findings (`total_errors`, `total_warnings`, `errors`, `warnings`, `notices`), percentages (`file_coverage`, `file_coverage_percent`, `files_covered`, `files_total`, `loc_coverage`, `loc_coverage_percent`, `loc_covered`, `loc_total`), results (`modules`, `uncovered_modules`, `uncovered_files`), and denominator provenance (`missing_files`, `skipped_links`, `manifest_notices`). A list that names only the percentage keys hides the findings a consumer needs to read.
