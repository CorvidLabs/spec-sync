---
change: give-the-macos-release-candidate-qualification-75-minutes-after-v6-2-0-rc-1-timed-out
artifact: plan
---

# Plan

## Approach

Raise only the `qualify` job's `timeout-minutes` in `.github/workflows/release.yml` from 45 to 75, and record v6.2.0-rc.1's macOS timeout in the 6.2.0 CHANGELOG section and release handoff.

## Out of scope

No runtime, dependency, spec, Action default, runner image or other workflow logic changes, and no tag, release or publication movement. rc.1 is not touched. Making the macOS lane itself faster is a separate issue.

## Steps

Edit the qualify job timeout; add the note to CHANGELOG 6.2.0 and docs/releases/6.2.0.md; run the three release validators; verify, review and finalize natively before merge; then tag v6.2.0-rc.2 on the merge commit if rc.1's re-run does not qualify.

## Risks

A longer limit lets a genuinely hung macOS job hold a runner for up to 75 minutes before it is cancelled. The other jobs' timeouts are unchanged.

## Constraints consulted

Repository AGENTS.md, docs/RELEASING.md and docs/releases/6.2.0.md. RC tags are immutable, so a failed candidate is superseded by the next N. 0xLeif chose to raise this timeout for 6.2.0 and track the slowness separately.
