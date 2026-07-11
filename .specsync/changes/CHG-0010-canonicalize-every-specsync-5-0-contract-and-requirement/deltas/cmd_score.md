## ADDED

### REQUIREMENT REQ-cmd-score-001

The score command SHALL produce deterministic per-spec and project quality scores while honoring filters, formats, and release gates.

Acceptance Criteria
- `cmd_score` scores discovered specs (after `filter_specs` and `filter_by_status`) using `score_spec`, then aggregates via `compute_project_score`
- Five dimensions, 20 points each: Frontmatter, Sections, API, Depth, Freshness
- JSON output includes per-spec objects (`total`, `grade`, the five sub-scores, `suggestions`) and a project object (`average_score` rounded to 1 dp, `grade`, `total_specs`, A–F `distribution`); `--explain` adds an `explain` array per spec
- `--format table` renders an aligned ASCII table; with `--explain` it adds FM/Sec/API/Depth/Fresh columns
- `--format csv` prints a header row, one row per spec, and a final `SUMMARY` row with the average, grade, and distribution
- Default/text output prints each spec's grade and either the 5-subscore line or, with `--explain`, a per-criterion breakdown with ✓/✗ marks and point details, followed by suggestions
- Batch mode (no filters, or `--all`) prints a "Scoring N spec(s)…" progress header in text mode (suppressed for JSON/CSV)
- Grades are colorized by band (A/B green, C/D yellow, F red); subscores colorized (20 green, 10–19 yellow, <20 red)

## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_score` | `root: &Path, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>, format: types::OutputFormat, explain: bool, all: bool, spec_filters: &[String], exclude_status: &[String], only_status: &[String]` | `()` | Score all or filtered specs and display grades |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs` |
| scoring | `score_spec`, `compute_project_score` |
| types | `OutputFormat` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync score` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/validator/validator.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
