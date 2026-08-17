---
spec: cmd_score.spec.md
---

## User Stories

- As a developer, I want `specsync score` to grade each spec 0–100 with a letter grade so that I know which docs need work
- As a developer, I want `--explain` to break the score into per-dimension, per-criterion lines so that I can see exactly why points were lost
- As a team lead, I want a project-level average and A–F distribution so that I can track overall spec health at a glance
- As a dashboard author, I want `--format csv` (with a SUMMARY row) and `--format json` so that I can feed scores into spreadsheets or tooling
- As a CI user, I want `--format table` for a compact aligned report so that scores read cleanly in logs
- As a developer, I want to scope scoring with positional spec filters and `--exclude-status`/`--only-status` so that I can focus on relevant specs

## Acceptance Criteria

- `cmd_score` scores discovered specs (after `filter_specs` and `filter_by_status`) using `score_spec`, then aggregates via `compute_project_score`
- Five dimensions, 20 points each: Frontmatter, Sections, API, Depth, Freshness
- JSON output includes per-spec objects (`total`, `grade`, the five sub-scores, `suggestions`) and a project object (`average_score` rounded to 1 dp, `grade`, `total_specs`, A–F `distribution`); `--explain` adds an `explain` array per spec
- `--format table` renders an aligned ASCII table; with `--explain` it adds FM/Sec/API/Depth/Fresh columns
- `--format csv` prints a header row, one row per spec, and a final `SUMMARY` row with the average, grade, and distribution
- Default/text output prints each spec's grade and either the 5-subscore line or, with `--explain`, a per-criterion breakdown with ✓/✗ marks and point details, followed by suggestions
- Batch mode (no filters, or `--all`) prints a "Scoring N spec(s)…" progress header in text mode (suppressed for JSON/CSV)
- Grades are colorized by band (A/B green, C/D yellow, F red); subscores colorized (20 green, 10–19 yellow, <20 red)
- Malformed Gradle/manifest discovery exits nonzero; JSON remains valid with `valid: false`, `inconclusive: true`, null score and grade, zero counts/distribution, empty specs, and an explicit error.

## Constraints

- Must not panic on expected error conditions — print and exit
- Scoring logic itself lives in the `scoring` module; this command only orchestrates and renders
- Status filtering reuses `filter_by_status`; unmatched positional filters print a warning via `filter_specs`
- Output must honor the requested `OutputFormat` (Text/Json/Table/Csv) — anything unrecognized falls through to text

## Out of Scope

- Defining or tuning the scoring rubric (owned by the scoring module)
- Failing the build on score quality alone — warn-mode scoring is advisory, while configured coverage gates and inconclusive discovery may exit nonzero
- Writing scores to a file or committing them
- Interactive prompts

### REQ-cmd-score-001

The `cmd_score` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-score-002

The `score` strict floor SHALL remain inclusive, and SHALL gate a directory mapping by that spec scoring zero rather than by changing the boundary.

Acceptance Criteria
- A total equal to the floor passes; only a total below it fails.
- A spec whose `files:` entry is a directory fails the floor because it scores zero, not because the comparison was made exclusive.
- `score` does not become a second `check`: it continues to report a metric for every spec it is given.

