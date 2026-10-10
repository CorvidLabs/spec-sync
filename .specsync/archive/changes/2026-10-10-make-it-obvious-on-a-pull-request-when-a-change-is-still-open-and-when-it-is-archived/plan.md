---
change: make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived
artifact: plan
---

# Plan

## Approach

Add an Archive job that always runs. It fails while any change workspace is still open, names the one to archive first, and gives one command. It passes only when every change is archived, and then it says the pull request is ready to merge when the other checks are green. The required CI gate includes this job, so an open change does not look finished.

`change ship`, once the change is ready, finalizes and commits the archive tip itself. Its next line says to push that commit, wait until the checks are green, and then merge. If another change is still open, that line names the one to archive next and does not say to merge.

`change status` prints the same sentence the Archive job prints.

## Out of scope

No automatic archive on push. Review stays a human step. No push inside `change ship` unless `--push` is passed. No change to how product tests, fmt, or spec-check are selected.

## Steps

1. Add the Archive job, the readiness script, and its test, and register the job on the required gate.
2. Make `change ship` commit the archive tip and say what is left in one line.
3. Make `change status` print the same sentence, including which open change comes first.
4. Dogfood the check on this pull request before anyone merges it.

## Risks

A required Archive job fails the rollup while a change is open, so a product tip is not all-green before the archive commit. That is the point of the check. The other jobs can still pass on their own.

Picking the wrong change first would stall a stack. The order is the state closest to archive, then the lowest id.

## Constraints consulted

Do not withhold a merge on an open change by hiding the fact. One sentence names the state. One command finishes the archive and commits it. Do not tell me to merge when it is not time.
