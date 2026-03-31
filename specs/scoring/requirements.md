---
spec: scoring.spec.md
---

## User Stories

- As a developer, I want each spec scored on a 0-100 scale with a letter grade so that I can quickly assess documentation quality
- As a team lead, I want a project-wide score with grade distribution so that I can track documentation health across the entire codebase
- As a developer, I want actionable improvement suggestions so that I know exactly what to fix to raise my score
- As a developer of a config-only module, I want to not be penalized for having no exports to document so that the scoring is fair

## Acceptance Criteria

- [ ] Five scoring components, 20 points each: frontmatter completeness, required sections, API documentation coverage, content depth, freshness
- [ ] Frontmatter scoring: module (5pts), version (5pts), status (4pts), non-empty files list (6pts)
- [ ] Content depth checks for meaningful content beyond headings and TODO comments
- [ ] Freshness deducts for stale file references (5pts each, max 15) and stale dependency refs (3pts each)
- [ ] Grade scale: A (90-100), B (80-89), C (70-79), D (60-69), F (<60)
- [ ] TODO counting ignores fenced code blocks
- [ ] Only counts actual TODOs, not compound terms containing "TODO" (e.g., "TODO-marker")
- [ ] Modules with no exports to document receive full API score (20/20)
- [ ] Suggestions are always actionable (e.g., "Add module: field to frontmatter", not just "frontmatter incomplete")
- [ ] `compute_project_score` produces an average score, overall grade, and per-grade distribution count

## Constraints

- Scoring must be deterministic — same spec always produces same score
- Must not make network calls or spawn processes
- Score breakdown must be transparent — each component clearly explained

## Out of Scope

- Scoring prose quality or readability (only structural completeness)
- Comparing scores across different projects
- Historical score tracking or trend analysis
- Weighting components differently per project
