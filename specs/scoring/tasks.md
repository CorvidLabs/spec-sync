---
spec: scoring.spec.md
---

## Tasks

- [ ] Add configurable scoring weights (allow projects to prioritize certain dimensions)
- [ ] Add historical score tracking (compare current scores against previous runs)
- [ ] Add per-section content quality heuristics (beyond just "has content")
- [ ] Support a minimum score threshold in CI (`--min-score 70`)

## Done

- [x] 5-component scoring rubric (frontmatter, sections, API, content, freshness)
- [x] Letter grade calculation (A-F)
- [x] TODO/placeholder detection with code block exclusion
- [x] Stale file and dependency reference penalties
- [x] Actionable improvement suggestions
- [x] Project-level score aggregation with grade distribution
- [x] No-export modules handled gracefully

## Gaps

- Scoring weights are hardcoded (20 each) — projects can't customize priorities
- No way to track score trends over time
- Content depth check is binary (has content or doesn't) — no nuance between thorough and minimal documentation

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
