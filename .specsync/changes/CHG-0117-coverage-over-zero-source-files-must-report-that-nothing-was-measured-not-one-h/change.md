---
id: CHG-0117-coverage-over-zero-source-files-must-report-that-nothing-was-measured-not-one-h
state: implementing
type: bug_fix
base_commit: a38c3554152b32867daf5df740c530d40aa221b5
---

# Coverage over zero source files must report that nothing was measured, not one hundred percent

## Intent

Coverage over zero source files must report that nothing was measured, not one hundred percent

## Affected Canonical Specs

- `output`

## Acceptance Criteria

- Coverage output over a project with no source files reports that there was nothing to measure rather than a percentage, and names the likely cause so the configuration can be corrected. The affirmative lines claiming every source file is referenced and every module has a spec directory are not printed when no source files were found, because both are true only of an empty set. A project that does contain source files reports its percentages and affirmative lines exactly as before. Gate behaviour is unchanged.

## No-spec Rationale

Not applicable
