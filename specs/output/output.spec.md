---
module: output
version: 8
status: stable
files:
  - src/output.rs
db_tables: []
tracks: []
depends_on:
  - specs/types/types.spec.md
  - specs/parser/parser.spec.md
  - specs/exports/exports.spec.md
---

# Output

## Purpose

Renders terminal and markdown output for spec-sync commands. Provides colored text summaries (check results, coverage reports) and structured markdown output (PR comments, drift reports). Centralizes all presentation formatting so command modules focus on logic, not display.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `print_summary` | `total, passed, warnings, _errors: usize` | `()` | Print colored one-line validation summary (green/yellow/red counts) |
| `eprint_summary` | `total, passed, warnings, errors: usize` | `()` | The same summary on stderr, for formats whose stdout is a machine protocol (CSV) |
| `print_coverage_line` | `coverage: &CoverageReport` | `()` | Print file and LOC coverage percentages with color thresholds, followed by the missing files and skipped symlinks that shaped the denominator |
| `eprint_coverage_line` | `coverage: &CoverageReport` | `()` | The same coverage figures on stderr, so a CSV run still shows a human the numbers without putting prose in the CSV |
| `print_coverage_report` | `coverage: &CoverageReport` | `()` | Print detailed coverage report: unspecced modules, uncovered files with LOC |
| `csv_field` | `value: &str` | `String` | Quote a CSV field containing a delimiter, quote, or newline — one implementation for every CSV renderer |
| `findings` | `errors, warnings, notices: &[String]` | `Vec<Finding>` | The full finding set in a stable order (errors, warnings, notices), split into severity/spec/message columns. Every format renders this one list |
| `print_findings_csv` | `findings: &[Finding]` | `()` | Render the findings as CSV: a stable header plus one row per finding, header printed even when there are none |
| `print_findings_table` | `findings: &[Finding]` | `()` | Render the findings as an aligned ASCII table, header printed even when there are none |
| `coverage_json` | `coverage: &CoverageReport, findings: &CoverageFindings` | `serde_json::Value` | THE constructor for every coverage payload (CLI `coverage --format json`, the `specsync_coverage` MCP tool, the `specsync:///coverage` MCP resource) — percentages, counts, and the finding set together |
| `print_check_markdown` | `total, passed, warnings, errors, all_errors, all_warnings, all_notices, coverage, overall_passed` | `()` | Print check results with separate error, warning, and planned-mapping notice sections as markdown |
| `print_diff_markdown` | `entries, changed_files, spec_files, _root, config, base` | `()` | Print drift report as markdown showing new/removed exports per spec since a base ref |
| `percent_json` | `value: Option<usize>` | `serde_json::Value` | Render a percentage for a JSON payload: the number, or `null` when nothing was measured. `null` rather than `0` so a consumer can tell an unmeasured tree from a genuinely uncovered one |
| `NO_FILES_MEASURED` | — | `&str` | The wording used in place of a file-coverage percentage when there are no source files to measure |
| `NO_LINES_MEASURED` | — | `&str` | The wording used in place of a LOC-coverage percentage when there are no source lines to measure |
| `FINDING_CSV_HEADER` | — | `&str` | The stable CSV column header for the findings renderer (`severity,spec,message`) |

**Exported Types**

| Type | Description |
|------|-------------|
| `Finding` | One validation finding split into `severity` / `spec` / `message`, the columns the tabular renderers need. Project-scoped findings carry an empty `spec` rather than being dropped |
| `CoverageFindings` | The validation findings that accompany a coverage payload. `passed` is the GATE VERDICT — the same boolean the exit code carries — and `errors`/`warnings`/`notices` answer the factual question independently of enforcement policy |

## Invariants

1. Color thresholds for coverage: 100% = green, 80-99% = yellow, <80% = red
2. `print_summary` counts: passed is green, warnings is yellow, failed is red (failed = total - passed)
3. `print_diff_markdown` calls into `parser::parse_frontmatter` and `exports::has_extension` to cross-reference changed files against spec source file lists
4. Markdown output uses GitHub-flavored markdown with tables and emoji status icons (✅/❌/⚠)
5. All functions write to stdout via `println!` — no buffered or file output

## Behavioral Examples

### Scenario: All specs pass

- **Given** 25 specs checked, 25 passed, 0 warnings
- **When** `print_summary(25, 25, 0, 0)` is called
- **Then** output: `25 specs checked: 25 passed, 0 warning(s), 0 failed` (25 in green, 0 in green)

### Scenario: Coverage below 80%

- **Given** coverage is 58% file coverage
- **When** `print_coverage_line()` is called
- **Then** percentage is displayed in red

### Scenario: Diff with no changes

- **Given** no spec-tracked source files changed since base ref
- **When** `print_diff_markdown()` is called with empty entries
- **Then** prints "No spec-tracked source files changed since `{base}`."

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty spec list | `print_summary` shows "0 passed, 0 failed" |
| Coverage report with no unspecced files | Shows "✓ All source files referenced by specs" |
| Diff with changed files not in any spec | Lists them under "Changed files not covered by any spec" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `CoverageReport`, `SpecSyncConfig`, `OutputFormat` |
| parser | `parse_frontmatter` (in diff markdown rendering) |
| exports | `has_extension` (to filter changed files by source extensions) |
| colored | Terminal color formatting |

### Consumed By

| Module | What is used |
|--------|-------------|
| cmd_check | `print_summary`, `print_coverage_line` |
| cmd_coverage | `print_coverage_line`, `print_coverage_report` |
| cmd_generate | `print_summary`, `print_coverage_line` |
| cmd_diff | `print_diff_markdown` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str: Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di: A symlink under a source directory must be skipped and disclosed, never abort discovery |
| 2026-08-14 | CHG-0117-coverage-over-zero-source-files-must-report-that-nothing-was-measured-not-one-h: Coverage over zero source files must report that nothing was measured, not one hundred percent |
| 2026-08-14 | CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac: Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question |
| 2026-08-14 | CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable: Every output format must report the same set of findings, so a machine-readable consumer cannot see fewer problems than a human reading the text |
