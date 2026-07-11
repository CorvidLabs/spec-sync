## ADDED

### REQUIREMENT REQ-output-001

Output helpers SHALL render consistent terminal and Markdown summaries without changing validation state or corrupting machine formats.

Acceptance Criteria
- `print_summary(total, passed, warnings, errors)` prints `{total} specs checked: {passed} passed, {warnings} warning(s), {failed} failed`, where `failed = total.saturating_sub(passed)` (never underflows)
- `passed` is colored green, `warnings` yellow, and `failed` red when nonzero (otherwise `0`)
- `print_coverage_line` colors both file and LOC percentages green at 100, yellow at ≥80, red below 80
- `print_coverage_report` prints a "✓ All source modules have spec directories" / "✓ All source files referenced by specs" success line when nothing is unspecced, otherwise lists each unspecced module (`name/`) and file (with per-file LOC and an uncovered-LOC total)
- `print_check_markdown` emits a `## SpecSync Check Results` block with a ✅/❌ status line, optional Errors/Warnings sections, and a Coverage section
- `print_diff_markdown` emits a `## SpecSync Drift Report`; with no drift entries it reports either "No spec-tracked source files changed since `{base}`." or lists changed files not covered by any spec; with entries it tabulates added/removed exports per spec and flags spec-file-only modifications
