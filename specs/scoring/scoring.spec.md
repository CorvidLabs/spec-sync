---
module: scoring
version: 7
status: stable
files:
  - src/scoring.rs
db_tables: []
tracks: [31]
depends_on:
  - specs/types/types.spec.md
  - specs/parser/parser.spec.md
  - specs/exports/exports.spec.md
  - specs/git_utils/git_utils.spec.md
---

# Scoring

## Purpose

Scores spec quality on a 0-100 scale with letter grades. Uses a 5-component rubric (20 points each): frontmatter completeness, required sections, API documentation coverage, content depth, and freshness. Provides actionable improvement suggestions.

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `SpecScore` | Quality score for a single spec: component scores, total, grade, and suggestions |
| `ProjectScore` | Aggregate scores for the project: average, grade, distribution, and per-spec scores |
| `CriterionResult` | Pass/fail result for a single scoring criterion within a dimension |
| `ExplainDetail` | Per-dimension breakdown (criteria list, score, max) used by `--explain` |

### Exported Enums

| Type | Description |
|------|-------------|
| `GitFreshness` | Whether the git-history half of the freshness dimension could be measured: `NotApplicable` (spec lists no files), `Measured`, or `Withheld` (no history existed, so the points were not awarded). Consumers that also confine git — the MCP snapshot — read it so one absence is not charged twice |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `score_spec` | `spec_path, root, config` | `SpecScore` | Score a single spec file on 5 quality dimensions |
| `compute_project_score` | `spec_scores: Vec<SpecScore>` | `ProjectScore` | Aggregate individual spec scores into a project-level summary |

## Invariants

1. Total score is always 0-100, composed of 5 components each worth 0-20 points
2. Grade scale: A (90-100), B (80-89), C (70-79), D (60-69), F (<60)
3. Frontmatter scoring: module (5pts), version (5pts), status (4pts), files non-empty (6pts)
4. Unfinished-work marker counting ignores occurrences inside fenced code blocks
5. Unfinished-work marker counting only counts standalone markers — not compound terms or descriptive prose
6. Content depth checks that sections have meaningful content beyond headings, comments, and separator rows
7. Freshness penalizes stale file references (5pts each, max 15pt penalty) and stale dependency refs (3pts each)
7a. The 5-point git-freshness budget is WITHHELD, never awarded, when there is no committed history to measure drift against — a spec's score can never rise because `.git` was removed or never created (#572). A repository that has history but has not committed this spec yet is a different case: drift there is genuinely zero, and the modification-time fallback stands in
8. Suggestions are always actionable — each corresponds to a specific improvement the user can make
9. No exports to document = full API score (20/20) — specs for config-only modules are not penalized
10. `SpecScore.explain` is always populated during `score_spec` — one `ExplainDetail` per dimension, each containing one or more `CriterionResult` entries

## Behavioral Examples

### Scenario: Perfect spec

- **Given** a spec with complete frontmatter, all sections present, 100% API coverage, no unfinished-work markers, all files exist
- **When** `score_spec` is called
- **Then** returns total=100, grade="A", empty suggestions

### Scenario: Skeleton spec with unfinished markers

- **Given** a spec with all sections but only unfinished-work markers in content
- **When** `score_spec` is called
- **Then** depth_score is low and suggestions identify the sections that need substantive content

### Scenario: Project score aggregation

- **Given** 3 specs scoring 95, 80, 65
- **When** `compute_project_score` is called
- **Then** average_score=80.0, grade="B", distribution shows 1 A, 1 B, 0 C, 1 D, 0 F

### Scenario: --explain breakdown

- **Given** a spec scoring 11/20 on Depth
- **When** `score_spec` is called
- **Then** `explain` contains a Depth entry with `CriterionResult` items showing which checks passed/failed

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Returns score=0, grade="F", suggestion: "Cannot read spec file" |
| Missing frontmatter | Returns score=0, grade="F", suggestion: "Add YAML frontmatter" |
| No spec files in project | `compute_project_score` returns average=0, grade="F" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| parser | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| exports | `get_exported_symbols` |
| types | `SpecSyncConfig` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `score_spec`, `compute_project_score` |
| mcp | `score_spec`, `compute_project_score` |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-07 | Replace template-marker suggestion wording with unfinished draft marker wording |
| 2026-04-18 | Add `CriterionResult` and `ExplainDetail` structs; add `explain` field to `SpecScore` for `--explain` breakdown |
| 2026-03-25 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-14 | CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i: Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension |
| 2026-08-14 | CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused: A source file or spec body carrying an unresolved merge conflict must be refused, because extracting declarations from both sides of a hunk describes source that does not exist |
