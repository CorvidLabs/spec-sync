## ADDED

### REQUIREMENT REQ-cmd-coverage-001

The coverage command SHALL report file and LOC coverage in human and machine formats and SHALL honor configured release gates.

Acceptance Criteria
- `cmd_coverage(root, strict, enforcement, require_coverage, format)` runs validation (`run_validation`) before computing coverage so results are consistent with `check`
- Coverage is computed via `compute_coverage`; file-coverage and loc-coverage percentages are derived and rounded to two decimals, treating a zero-denominator as 100%
- JSON output (`--format json`) emits `file_coverage`, `files_covered`, `files_total`, `loc_coverage`, `loc_covered`, `loc_total`, `modules` (unspecced), and `uncovered_files`, then exits 0
- Non-JSON output prints the coverage report, the validation summary, and a coverage line, then delegates the exit code to `exit_with_status`
- `--enforcement` overrides config enforcement; `--strict` implies strict enforcement; `--require-coverage N` fails (exit 1) when coverage is below N
