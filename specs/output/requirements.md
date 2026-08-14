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

The `output` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-output-002

Markdown check output SHALL accept planned-mapping notices and render a distinct Planned Mappings section.

Acceptance Criteria

- The canonical `print_check_markdown` signature includes the notice collection.
- Planned mappings are separate from errors and warnings.
- The notice section does not alter validation state or pass/fail decisions.

### REQ-output-003

Coverage output SHALL disclose skipped symlinked entries alongside the coverage figures.

Acceptance Criteria
- Text output names the skipped entries immediately after the coverage lines.
- Markdown output names them within the coverage section.
- A fixed number of entries are named explicitly and any remainder is summarized with a count.
- Output with no skipped entries is unchanged.

### REQ-output-004

Coverage output SHALL NOT report a percentage when there was nothing to measure, and SHALL
NOT make affirmative claims that are true only of an empty set.

Acceptance Criteria
- A zero file denominator reports that there were no source files to measure, rather than a percentage.
- A zero line denominator reports that there were no source lines to measure, rather than a percentage.
- When no source files were found, the claims that every source file is referenced and every module has a spec directory are not printed.
- In their place the likely cause is named, so a misconfigured source directory or an over-broad exclusion can be corrected.
- A project containing source files reports its percentages and affirmative lines unchanged.

### REQ-output-005

Text output SHALL state that nothing was measured rather than print a percentage.

Acceptance Criteria
- A zero denominator prints the measured counts and names the reason.
- The renderer derives from the shared accessor rather than re-computing the ratio.
