## ADDED

### REQUIREMENT REQ-output-002

Markdown check output SHALL accept planned-mapping notices and render a distinct Planned Mappings section.

Acceptance Criteria

- The canonical `print_check_markdown` signature includes the notice collection.
- Planned mappings are separate from errors and warnings.
- The notice section does not alter validation state or pass/fail decisions.


## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `print_summary` | `total, passed, warnings, _errors: usize` | `()` | Print colored one-line validation summary (green/yellow/red counts) |
| `print_coverage_line` | `coverage: &CoverageReport` | `()` | Print file and LOC coverage percentages with color thresholds |
| `print_coverage_report` | `coverage: &CoverageReport` | `()` | Print detailed coverage report: unspecced modules, uncovered files with LOC |
| `print_check_markdown` | `total, passed, warnings, errors, all_errors, all_warnings, all_notices, coverage, overall_passed` | `()` | Print check results with separate error, warning, and planned-mapping notice sections as markdown |
| `print_diff_markdown` | `entries, changed_files, spec_files, _root, config, base` | `()` | Print drift report as markdown showing new/removed exports per spec since a base ref |
