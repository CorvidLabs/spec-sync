---
id: the-watch-tests-read-a-run-through-to-the-line-after-its-verdict
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The watch tests read a run through to the line after its verdict

## Intent

The watch tests read a run through to the line after its verdict

## Affected Canonical Specs

- None

## Acceptance Criteria

- watch_until stops only once a verdict marker is followed by the line watch prints after each run (Watching for changes in: after the first, Watching for changes... after later ones); a unit test on sample output fails on the verdict alone and passes with the line after it (red then green)

## No-spec Rationale

A test-only fix: watch_until stopped at the first verdict marker, so a later line of the same run, such as a contradicting warning, was never in the output the watch tests judge. No spec or requirement text changes.
