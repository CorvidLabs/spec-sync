---
spec: cmd_coverage.spec.md
---

## User Stories

- As a developer, I want a file-level and LOC-level spec coverage report so I can see which source files are not yet claimed by any spec
- As a CI operator, I want `--require-coverage N` (and `--strict`/`--enforcement`) to fail the build when coverage drops below a threshold so coverage regressions are caught
- As a tooling author, I want `--format json` to emit machine-readable coverage metrics (percentages, counts, uncovered files, unspecced modules) so I can integrate with dashboards

## Acceptance Criteria

- `cmd_coverage(root, strict, enforcement, require_coverage, format)` runs validation (`run_validation`) before computing coverage so results are consistent with `check`
- Coverage is computed via `compute_coverage_checked`; file-coverage and loc-coverage percentages are derived and rounded to two decimals, treating a trustworthy zero-denominator as 100%
- JSON output emits coverage metrics and honors gates; malformed discovery exits 1 with valid JSON containing `valid: false`, `inconclusive: true`, null percentages, zero counts, empty collections, and an explicit error
- Non-JSON output prints the coverage report, the validation summary, and a coverage line, then delegates the exit code to `exit_with_status`
- `--enforcement` overrides config enforcement; `--strict` implies strict enforcement; `--require-coverage N` fails (exit 1) when coverage is below N

## Constraints

- Coverage computation, validation, and exit-status logic all live in shared modules (`validator`, `commands` helpers); this wrapper orchestrates and formats
- JSON remains machine-readable while honoring validation, enforcement, threshold, and inconclusive-discovery failures
- This command uses `IgnoreRules::default()` (no `.specsyncignore` loading), unlike `check`/`comment` which load ignore rules from disk

## Out of Scope

- The coverage algorithm itself (owned by `validator::compute_coverage_checked`)
- Writing the report to a file (output goes to stdout)
- Interactive prompts or GUI

### REQ-cmd-coverage-001

The coverage command SHALL report file and LOC coverage in human and machine formats and SHALL honor configured release gates.

Acceptance Criteria
- `cmd_coverage(root, strict, enforcement, require_coverage, format)` runs validation (`run_validation`) before computing coverage so results are consistent with `check`
- Coverage is computed via `compute_coverage_checked`; file-coverage and loc-coverage percentages are derived and rounded to two decimals when discovery succeeds
- JSON output emits normal coverage metrics on success and remains valid on malformed discovery with `valid: false`, `inconclusive: true`, null percentages, zero counts, empty collections, and an explicit error
- Non-JSON output prints the coverage report, the validation summary, and a coverage line, then delegates the exit code to `exit_with_status`
- `--enforcement` overrides config enforcement; `--strict` implies strict enforcement; `--require-coverage N` fails (exit 1) when coverage is below N
- Malformed Gradle/manifest discovery exits nonzero in every format rather than producing partial or vacuous coverage.

