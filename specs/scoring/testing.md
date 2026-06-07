---
spec: scoring.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/scoring.rs` inline tests | Unit | Validate Scoring behavior close to implementation, especially `SpecScore`, `ProjectScore`, `CriterionResult`, `ExplainDetail`, `score_spec`, `compute_project_score`, `get_exported_symbols`, `SpecSyncConfig` |
| `tests/integration.rs` | Integration | Exercise Scoring through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Scoring contracts or source files.
- [ ] Run `fledge run test` and confirm Scoring unit/integration coverage still passes.
- [ ] Review examples in `scoring.spec.md` against observed behavior when touching src/scoring.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec file unreadable | Returns score=0, grade="F", suggestion: "Cannot read spec file" |
| Missing frontmatter | Returns score=0, grade="F", suggestion: "Add YAML frontmatter" |
| No spec files in project | `compute_project_score` returns average=0, grade="F" |
