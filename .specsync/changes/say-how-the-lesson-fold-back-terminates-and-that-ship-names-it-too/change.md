---
id: say-how-the-lesson-fold-back-terminates-and-that-ship-names-it-too
state: implementing
type: documentation
base_commit: 875752ee991d458db172dec6ceb712462fe2a614
---

# Say how the lesson fold-back terminates, and that ship names it too

## Intent

Say how the lesson fold-back terminates, and that ship names it too

## Affected Canonical Specs

- None

## Acceptance Criteria

- docs/ADOPTING.md's 'Close the learning loop' section states that the fold-back is itself a change, that scoping it to the modules it edits reproduces the fold-back instruction, and that a fold change declaring no affected specs terminates it; it gives a copy-pasteable 'change new --kind documentation --no-spec-change' command with rationale wording; and the section names both finalize and ship as the verbs that print the step, not finalize alone.

## No-spec Rationale

Documentation only: adds fold-back termination guidance to docs/ADOPTING.md and corrects a stale claim that only finalize names the step; no canonical spec text, requirement, or behaviour changes.
