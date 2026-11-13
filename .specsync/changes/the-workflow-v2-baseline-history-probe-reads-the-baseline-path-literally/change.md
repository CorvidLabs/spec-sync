---
id: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The workflow-v2 baseline history probe reads the baseline path literally

## Intent

The workflow-v2 baseline history probe reads the baseline path literally

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- in a project directory named :(exclude)app with no committed baseline, the reachable-history probe answers false and reading the baseline gives none instead of reporting it deleted

## No-spec Rationale

A baseline that was never committed is absent, not deleted; the reachable-history probe passed the baseline path as a plain pathspec, so a project directory named like pathspec magic read as an exclude and reported the baseline deleted. The module's other history queries are literal top-level pathspecs (#851, #990) (#1074)
