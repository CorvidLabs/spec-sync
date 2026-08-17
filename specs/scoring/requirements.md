---
spec: scoring.spec.md
---

## User Stories

- As a developer, I want each spec scored on a 0-100 scale with a letter grade so that I can quickly assess documentation quality
- As a team lead, I want a project-wide score with grade distribution so that I can track documentation health across the entire codebase
- As a developer, I want actionable improvement suggestions so that I know exactly what to fix to raise my score
- As a developer of a config-only module, I want to not be penalized for having no exports to document so that the scoring is fair

## Acceptance Criteria

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

## Constraints

- Scoring must be deterministic — same spec always produces same score
- Must not make network calls or spawn processes
- Score breakdown must be transparent — each component clearly explained

## Out of Scope

- Scoring prose quality or readability (only structural completeness)
- Comparing scores across different projects
- Historical score tracking or trend analysis
- Weighting components differently per project

### REQ-scoring-001

The `scoring` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.


### REQ-scoring-002

The freshness dimension SHALL withhold points it could not measure.

Acceptance Criteria
- Unmeasurable git freshness withholds its points rather than awarding them.
- Whether the git half was measured, not applicable, or withheld is recorded so consumers do not withhold twice.
- Removing git history cannot raise a spec's grade.

### REQ-scoring-003

A spec with a conflicted source file SHALL NOT be awarded API credit.

Acceptance Criteria
- API credit is withheld when any mapped file's extraction unioned both sides of a conflict, because the union describes a tree that does not compile.
- The withholding applies even when the spec's other files parsed cleanly: scoring the readable remainder would report a confident number over an uncompilable tree.
- The reason is explained in the score breakdown rather than presented as a low score with no cause.

### REQ-scoring-004

The API dimension SHALL grade against the configured export surface.

Acceptance Criteria
- `score` and `check` never disagree about which symbols constitute a module's API.
- A symbol outside the configured surface is neither counted nor named as undocumented.
- A project on the default surface scores exactly as before.

### REQ-scoring-005

A spec whose `files:` entry is a directory SHALL score zero and name the directory, rather than scoring as a merely incomplete spec.

Acceptance Criteria
- The freshness dimension fails and names the directory, because a directory is not an existing source file.
- The API dimension is zero and names the directory, rather than reporting the path as missing or not valid UTF-8.
- The spec total is zero with grade F, which is below every strict and minimum-score floor including the inclusive eighty-point bar.
- Scoring remains a metric rather than a hard failure, so explain and machine-readable output still render for the affected spec.
- A spec naming a real source file is scored exactly as before, so the rule cannot be satisfied by lowering scores generally.

