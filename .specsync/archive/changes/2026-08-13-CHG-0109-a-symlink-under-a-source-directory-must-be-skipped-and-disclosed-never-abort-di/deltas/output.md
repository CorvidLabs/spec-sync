## ADDED

### REQUIREMENT REQ-output-003

Coverage output SHALL disclose skipped symlinked entries alongside the coverage figures.

Acceptance Criteria
- Text output names the skipped entries immediately after the coverage lines.
- Markdown output names them within the coverage section.
- A fixed number of entries are named explicitly and any remainder is summarized with a count.
- Output with no skipped entries is unchanged.

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

