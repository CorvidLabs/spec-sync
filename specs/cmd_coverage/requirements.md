---
spec: cmd_coverage.spec.md
---

## User Stories

- As a developer, I want a file-level and LOC-level spec coverage report so I can see which source files are not yet claimed by any spec
- As a CI operator, I want `--require-coverage N` (and `--strict`/`--enforcement`) to fail the build when coverage drops below a threshold so coverage regressions are caught
- As a tooling author, I want `--format json` to emit machine-readable coverage metrics (percentages, counts, uncovered files, unspecced modules) so I can integrate with dashboards

## Acceptance Criteria

- `cmd_coverage(root, strict, enforcement, require_coverage, format)` runs validation (`run_validation`) before computing coverage so results are consistent with `check`
- Coverage is computed via `compute_coverage`; file-coverage and loc-coverage percentages are derived and rounded to two decimals, treating a zero-denominator as 100%
- JSON output (`--format json`) emits `file_coverage`, `files_covered`, `files_total`, `loc_coverage`, `loc_covered`, `loc_total`, `modules` (unspecced), and `uncovered_files`, then exits 0
- Non-JSON output prints the coverage report, the validation summary, and a coverage line, then delegates the exit code to `exit_with_status`
- `--enforcement` overrides config enforcement; `--strict` implies strict enforcement; `--require-coverage N` fails (exit 1) when coverage is below N

## Constraints

- Coverage computation, validation, and exit-status logic all live in shared modules (`validator`, `commands` helpers); this wrapper orchestrates and formats
- JSON path always exits 0 regardless of validation status (it is a metrics dump, not a gate)
- This command uses `IgnoreRules::default()` (no `.specsyncignore` loading), unlike `check`/`comment` which load ignore rules from disk

## Out of Scope

- The coverage algorithm itself (owned by `validator::compute_coverage`)
- Writing the report to a file (output goes to stdout)
- Interactive prompts or GUI

### REQ-cmd-coverage-001

The coverage command SHALL report file and LOC coverage in human and machine formats and SHALL honor configured release gates.

Acceptance Criteria
- `cmd_coverage(root, strict, enforcement, require_coverage, format)` runs validation (`run_validation`) before computing coverage so results are consistent with `check`
- Coverage is computed via `compute_coverage`; file-coverage and loc-coverage percentages are derived and rounded to two decimals, treating a zero-denominator as 100%
- JSON output (`--format json`) emits `file_coverage`, `files_covered`, `files_total`, `loc_coverage`, `loc_covered`, `loc_total`, `modules` (unspecced), and `uncovered_files`, then exits 0
- Non-JSON output prints the coverage report, the validation summary, and a coverage line, then delegates the exit code to `exit_with_status`
- `--enforcement` overrides config enforcement; `--strict` implies strict enforcement; `--require-coverage N` fails (exit 1) when coverage is below N

