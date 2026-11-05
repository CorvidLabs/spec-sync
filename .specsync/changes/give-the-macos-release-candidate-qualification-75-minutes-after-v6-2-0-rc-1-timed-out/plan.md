---
change: give-the-macos-release-candidate-qualification-75-minutes-after-v6-2-0-rc-1-timed-out
artifact: plan
---

# Plan

## Approach


## Out of scope


## Steps


## Risks


## Constraints consulted

1. Raise the `qualify` job's `timeout-minutes` in `.github/workflows/release.yml` from 45 to 75.
2. Record rc.1's timeout and this change in CHANGELOG.md and docs/releases/6.2.0.md.
3. Track the macOS lane slowness in a separate issue.
