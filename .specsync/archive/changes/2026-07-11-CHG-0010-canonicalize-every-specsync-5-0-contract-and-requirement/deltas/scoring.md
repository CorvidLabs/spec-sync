## ADDED

### REQUIREMENT REQ-scoring-001

Spec scoring SHALL assign transparent deterministic component scores, actionable suggestions, grades, and project distributions.

Acceptance Criteria
- Five scoring components, 20 points each: frontmatter completeness, required sections, API documentation coverage, content depth, freshness
- Frontmatter scoring: module (5pts), version (5pts), status (4pts), non-empty files list (6pts)
- Content depth checks for meaningful content beyond headings and unfinished-work comments
- Freshness deducts for stale file references (5pts each, max 15) and stale dependency refs (3pts each)
- Grade scale: A (90-100), B (80-89), C (70-79), D (60-69), F (<60)
- Unfinished-work marker counting ignores fenced code blocks
- Only counts standalone unfinished-work markers, not compound terms or descriptive prose
- Modules with no exports to document receive full API score (20/20)
- Suggestions are always actionable (e.g., "Add module: field to frontmatter", not just "frontmatter incomplete")
- `compute_project_score` produces an average score, overall grade, and per-grade distribution count
