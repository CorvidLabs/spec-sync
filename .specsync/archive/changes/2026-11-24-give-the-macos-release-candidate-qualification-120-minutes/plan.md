---
change: give-the-macos-release-candidate-qualification-120-minutes
artifact: plan
---

# Plan

## Approach

Change one value. `timeout-minutes` for the `qualify` job goes from 75 to 120, and the comment above it says what the lane needs now and where the real fix is tracked. Add a short paragraph to the 6.4.2 handoff.

## Out of scope

Making the lane faster, splitting it into parallel jobs, and setting limits to 30 minutes. Those are #1161.

## Steps

1. Edit `.github/workflows/release.yml` and `docs/releases/6.4.2.md`.
2. Run the version validator, the workflow-pin validator, the release-candidate contract tests and the required-gate tests.
3. Scoped check, strict check, pre-push and Trust. No native Verify lane: no product, test or spec file changes, and hosted CI runs the full suite on the pull request.
4. Review, archive, pull request, merge, then `v6.4.2-rc.2` on the merged main commit.

## Risks

A larger limit lets a hung job run longer before it is cancelled. 120 minutes leaves about 35 minutes over what the lane needs today, and the limit is meant to come down, not go up again.

## Constraints consulted

docs/RELEASING.md: a changed candidate tree needs a new candidate number, so this ships as `v6.4.2-rc.2`. SPEED-2.b.1 and SPEED-2.b.2.
