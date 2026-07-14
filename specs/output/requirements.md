---
spec: output.spec.md
---

## User Stories

- As a developer running `specsync check`, I want a colored one-line summary of passed/warning/failed counts so that I can see project health at a glance
- As a developer, I want file and LOC coverage printed with color thresholds (green at 100%, yellow ≥80%, red below) so that gaps stand out
- As a CI operator, I want check results and drift reports rendered as Markdown so that they can be posted to PR comments and job summaries
- As a developer, I want the coverage report to list unspecced modules and files (with uncovered LOC) so that I know exactly what to document next

## Acceptance Criteria

- `print_summary(total, passed, warnings, errors)` prints `{total} specs checked: {passed} passed, {warnings} warning(s), {failed} failed`, where `failed = total.saturating_sub(passed)` (never underflows)
- `passed` is colored green, `warnings` yellow, and `failed` red when nonzero (otherwise `0`)
- `print_coverage_line` colors both file and LOC percentages green at 100, yellow at ≥80, red below 80
- `print_coverage_report` prints a "✓ All source modules have spec directories" / "✓ All source files referenced by specs" success line when nothing is unspecced, otherwise lists each unspecced module (`name/`) and file (with per-file LOC and an uncovered-LOC total)
- `print_check_markdown` emits a `## SpecSync Check Results` block with a ✅/❌ status line, optional Errors/Warnings sections, and a Coverage section
- `print_diff_markdown` emits a `## SpecSync Drift Report`; with no drift entries it reports either "No spec-tracked source files changed since `{base}`." or lists changed files not covered by any spec; with entries it tabulates added/removed exports per spec and flags spec-file-only modifications

## Constraints

- Pure presentation layer — formats data passed in, no config loading, file I/O, or process exit
- Uses the `colored` crate for ANSI coloring; must not panic on any count combination (including `passed > total`)
- Markdown output must be plain (no ANSI) so it renders correctly in PR comments and CI summaries

## Out of Scope

- GUI or web interface
- Interactive prompts
- Deciding pass/fail or exit codes (callers in the command layer own that)
- Computing coverage or diff data (this module only renders it)

### REQ-output-001

Output helpers SHALL render consistent terminal and Markdown summaries without changing validation state or corrupting machine formats.

Acceptance Criteria
- `print_summary(total, passed, warnings, errors)` prints `{total} specs checked: {passed} passed, {warnings} warning(s), {failed} failed`, where `failed = total.saturating_sub(passed)` (never underflows)
- `passed` is colored green, `warnings` yellow, and `failed` red when nonzero (otherwise `0`)
- `print_coverage_line` colors both file and LOC percentages green at 100, yellow at ≥80, red below 80
- `print_coverage_report` prints a "✓ All source modules have spec directories" / "✓ All source files referenced by specs" success line when nothing is unspecced, otherwise lists each unspecced module (`name/`) and file (with per-file LOC and an uncovered-LOC total)
- `print_check_markdown` emits a `## SpecSync Check Results` block with a ✅/❌ status line, optional Errors/Warnings sections, and a Coverage section
- `print_diff_markdown` emits a `## SpecSync Drift Report`; with no drift entries it reports either "No spec-tracked source files changed since `{base}`." or lists changed files not covered by any spec; with entries it tabulates added/removed exports per spec and flags spec-file-only modifications

### REQ-output-002

Markdown check output SHALL accept planned-mapping notices and render a distinct Planned Mappings section.

Acceptance Criteria

- The canonical `print_check_markdown` signature includes the notice collection.
- Planned mappings are separate from errors and warnings.
- The notice section does not alter validation state or pass/fail decisions.

