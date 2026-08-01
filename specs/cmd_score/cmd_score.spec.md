---
module: cmd_score
version: 6
status: stable
files:
  - src/commands/score.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/scoring/scoring.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Score

## Purpose

Implements the `specsync score` command. Scores spec quality 0-100 (graded A-F) across five categories: frontmatter, sections, API, depth, freshness. Shows per-spec and project-level scores.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_score` | `root: &Path, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>, format: types::OutputFormat, explain: bool, all: bool, spec_filters: &[String], exclude_status: &[String], only_status: &[String]` | `()` | Score all or filtered specs and display grades |

## Invariants

5. Checked manifest discovery must succeed before coverage gates are evaluated; malformed Gradle
   settings are inconclusive and exit 1 even though ordinary scoring is advisory.

## Behavioral Examples

### Scenario: Score with explain

- **Given** `--explain` set
- **When** `cmd_score` runs
- **Then** each spec shows FM/Sec/API/Depth/Fresh subscores

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No specs match filters | Warning printed |
| Malformed Gradle settings prevent coverage discovery | Exits 1; JSON remains valid with `valid: false`, `inconclusive: true`, null score/grade, zero distribution, empty specs, and an explicit error |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs` |
| scoring | `score_spec`, `compute_project_score` |
| types | `OutputFormat` |

| validator | `compute_coverage_checked` |
**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync score` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/validator/validator.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-22 | v3: fail closed when malformed Gradle/manifest discovery makes score coverage gates inconclusive |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
