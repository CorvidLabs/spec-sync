---
hi: 1
families: [COVERAGE]
---

# Knowing where the gaps are

## Intent

Having a contract and having a good contract are different claims, and both should be measurable. I should be able to see what share of the codebase is described at all, which documents are thin, and which have been left behind by the code they describe. The numbers should be honest enough to put a gate on, and specific enough to tell me where the next hour is best spent.

## Criteria

- **COVERAGE-1**  I can see what share of my source has a contract at all, counted by file and by line.
  - **COVERAGE-1.a**  Uncovered files come back biggest first, so I know where the next hour is best spent.
  - **COVERAGE-1.b**  The count measures what the project actually ships rather than its tests and generated code.
- **COVERAGE-2**  A pipeline can refuse the work when coverage falls below the bar the team set.
- **COVERAGE-3**  Every contract is scored for quality, so "we have a spec" and "we have a good spec" stay different claims.
  - **COVERAGE-3.a**  The score comes with a grade I can read at a glance.
  - **COVERAGE-3.b**  The score shows exactly which points were lost and why.
  - **COVERAGE-3.c**  A pipeline can fail when any contract drops below the floor the team set.
- **COVERAGE-4**  A contract whose source has moved on without it is flagged, with how far behind it has fallen.
- **COVERAGE-5**  One report puts coverage, staleness and validation side by side per module, so I can read the project's health at a glance.
