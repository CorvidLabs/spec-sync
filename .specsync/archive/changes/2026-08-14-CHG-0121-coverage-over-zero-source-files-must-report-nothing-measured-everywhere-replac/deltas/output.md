## ADDED

### REQUIREMENT REQ-output-005

Text output SHALL state that nothing was measured rather than print a percentage.

Acceptance Criteria
- A zero denominator prints the measured counts and names the reason.
- The renderer derives from the shared accessor rather than re-computing the ratio.

## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `print_summary` | `total, passed, warnings, _errors: usize` | `()` | Print colored one-line validation summary (green/yellow/red counts) |
| `print_coverage_line` | `coverage: &CoverageReport` | `()` | Print file and LOC coverage percentages with color thresholds |
| `print_skipped_links` | `coverage: &CoverageReport` | `()` | Print the symlinked entries discovery skipped, immediately after the coverage figures so the percentages are never read without what was excluded from them |
| `print_coverage_report` | `coverage: &CoverageReport` | `()` | Print detailed coverage report: unspecced modules, uncovered files with LOC |
| `print_check_markdown` | `total, passed, warnings, errors, all_errors, all_warnings, all_notices, coverage, overall_passed` | `()` | Print check results with separate error, warning, and planned-mapping notice sections as markdown |
| `print_diff_markdown` | `entries, changed_files, spec_files, _root, config, base` | `()` | Print drift report as markdown showing new/removed exports per spec since a base ref |
| `percent_json` | `value: Option<usize>` | `serde_json::Value` | Render a percentage for a JSON payload: the number, or `null` when nothing was measured. `null` rather than `0` so a consumer can tell an unmeasured tree from a genuinely uncovered one |
| `NO_FILES_MEASURED` | — | `&str` | The wording used in place of a file-coverage percentage when there are no source files to measure |
| `NO_LINES_MEASURED` | — | `&str` | The wording used in place of a LOC-coverage percentage when there are no source lines to measure |

