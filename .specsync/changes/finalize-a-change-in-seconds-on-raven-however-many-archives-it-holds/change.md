---
id: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds
state: approved
type: bug_fix
base_commit: df7bace2742ff5910d9b6747ecfda7e639848c7e
---

# Finalize a change in seconds on Raven however many archives it holds

## Intent

Finalize a change in seconds on Raven however many archives it holds

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- SpecSync archive (finalize) no longer takes minutes on Raven: the Git processes one finalize spawns grow by a small bounded number per other archived change instead of re-walking every archive's history, and every verdict and refusal is identical to a live run.

## No-spec Rationale

Not applicable
