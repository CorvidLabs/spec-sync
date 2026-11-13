---
id: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Prefetch workers inherit the cutoff verdicts the command thread checked and persisted

## Intent

Prefetch workers inherit the cutoff verdicts the command thread checked and persisted

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- after a cold archived status whose archives are checked by prefetch workers, the persistent read cache holds the cutoff verdict of every workflow-v1 archive; verdicts and refusals are unchanged

## No-spec Rationale

REQ-change-136 says whether a workflow-v1 change existed at the trusted pre-v2 cutoff is read from the cutoff tree once per command and kept in the persistent read cache; prefetch workers each read the cutoff tree again and kept nothing, because only the command thread reaches the cache (#978)
