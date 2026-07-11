---
module: cmd_score
version: 2
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

1. Five categories, 20 points each
2. Grades: A (90+), B (80-89), C (70-79), D (60-69), F (<60)
3. `--explain` shows per-category breakdown
4. JSON includes per-spec objects and project aggregate

## Behavioral Examples

### Scenario: Score with explain

- **Given** `--explain` set
- **When** `cmd_score` runs
- **Then** each spec shows FM/Sec/API/Depth/Fresh subscores

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No specs match filters | Warning printed |

## Dependencies

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

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
