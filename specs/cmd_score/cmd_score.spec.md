---
module: cmd_score
version: 3
status: stable
files:
  - src/commands/score.rs
db_tables: []
tracks: [421]
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
| `cmd_score` | `root: &Path, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>, min_score: Option<u32>, format: types::OutputFormat, explain: bool, all: bool, spec_filters: &[String], exclude_status: &[String], only_status: &[String]` | `()` | Score selected specs and enforce an optional per-spec minimum; strict mode implies at least 80 |

## Invariants

1. Five categories, 20 points each
2. Grades: A (90+), B (80-89), C (70-79), D (60-69), F (<60)
3. `--explain` shows per-category breakdown
4. JSON includes per-spec objects and project aggregate
5. `--min-score N` fails when any selected spec is below N; `--strict` implies a minimum of 80 and cannot be weakened by a lower explicit value
6. JSON reports `minimum_score` and `gate_passed`; JSON/CSV remain parseable on gate failure

## Behavioral Examples

### Scenario: Score with explain

- **Given** `--explain` set
- **When** `cmd_score` runs
- **Then** each spec shows FM/Sec/API/Depth/Fresh subscores

### Scenario: CI quality gate

- **Given** an untouched generated scaffold scores below 80
- **When** `score --min-score 80` or `score --strict` runs
- **Then** the process exits 1 and JSON reports `gate_passed: false`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No specs match filters | Warning printed |
| Minimum outside 0-100 | Clap usage error, exit 2 |
| Minimum requested but no specs scored | Gate fails, exit 1 |

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
| 2026-07-26 | v3 / #421: add `--min-score`, make strict imply 80, and expose parseable JSON gate fields |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
