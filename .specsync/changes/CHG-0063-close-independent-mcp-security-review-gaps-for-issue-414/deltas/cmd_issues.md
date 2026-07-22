## MODIFIED

### REQUIREMENT REQ-cmd-issues-001

The issues command SHALL verify tracked GitHub references and SHALL report valid, closed, missing,
and unverifiable states predictably.

Acceptance Criteria

- References from all specs are verified in one globally deduplicated batch of at most 100 unique
  issue IDs.
- Confirmed issues are classified as valid, closed, or not_found; repository, authentication,
  transport, timeout, and malformed-provider failures are errors.
- Any batch/provider error contributes to the existing non-zero command outcome.
- Human-readable output uses gathered-reference count to distinguish an empty project from an
  all-error batch, and the latter summary includes its error count.
- Repository/provider resolution occurs only after a non-empty reference set is gathered; empty
  projects succeed without GitHub configuration or access.

### SPEC SECTION Invariants

1. The command gathers all `implements` and `tracks` references before repository/provider access.
2. Project-wide verification is globally deduplicated, capped, and time-bounded by the GitHub
   module.
3. Inconclusive provider outcomes remain errors and cannot become successful not_found results.
4. No-reference guidance is emitted only when no spec references were gathered.
5. An empty reference set performs no repository or provider resolution.
