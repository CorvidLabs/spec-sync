---
module: scoring
version: 1
status: stable
files:
  - src/scoring.rs
db_tables: []
depends_on:
  - specs/types/types.spec.md
  - specs/parser/parser.spec.md
  - specs/exports/exports.spec.md
---

# Scoring

## Purpose

Scores spec quality on a 0-100 scale with letter grades. Uses a 5-component rubric (20 points each): frontmatter completeness, required sections, API documentation coverage, content depth, and freshness. Provides actionable improvement suggestions.

## Requirements

### User Stories

- As a [role], I want [feature] so that [benefit]

### Acceptance Criteria

- [ ] <!-- TODO: define acceptance criteria -->

## Public API

### Exported Structs

| Type | Description |
|------|-------------|
| `SpecScore` | Quality score for a single spec: component scores, total, grade, and suggestions |
| `ProjectScore` | Aggregate scores for the project: average, grade, distribution, and per-spec scores |

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `score_spec` | `spec_path, root, config` | `SpecScore` | Score a single spec file on 5 quality dimensions |
| `compute_project_score` | `spec_scores: Vec<SpecScore>` | `ProjectScore` | Aggregate individual spec scores into a project-level summary |

## Invariants

1. Total score is always 0-100, composed of 5 components each worth 0-20 points
2. Grade scale: A (90-100), B (80-89), C (70-79), D (60-69), F (<60)
3. Frontmatter scoring: module (5pts), version (5pts), status (4pts), files non-empty (6pts)
4. TODO counting ignores occurrences inside fenced code blocks
5. TODO counting only counts actual placeholder TODOs — not compound terms like "TODO-marker" or descriptive prose
6. Content depth checks that sections have meaningful content beyond headings, comments, and separator rows
7. Freshness penalizes stale file references (5pts each, max 15pt penalty) and stale dependency refs (3pts each)
8. Suggestions are always actionable — each corresponds to a specific improvement the user can make
9. No exports to document = full API score (20/20) — specs for config-only modules aren't penalized

## Behavioral Examples

### Scenario: Perfect spec

- **Given** a spec with complete frontmatter, all sections present, 100% API coverage, no TODOs, all files exist
- **When** `score_spec` is called
- **Then** returns total=100, grade="A", empty suggestions

### Scenario: Skeleton spec with TODOs

- **Given** a spec with all sections but only TODO placeholders in content
- **When** `score_spec` is called
- **Then** depth_score is low, suggestions include "Fill in N TODO placeholder(s)"

### Scenario: Project score aggregation

- **Given** 3 specs scoring 95, 80, 65
- **When** `compute_project_score` is called
- **Then** average_score=80.0, grade="B", distribution shows 1 A, 1 B, 0 C, 1 D, 0 F

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
| 2026-03-25 | Initial spec |
