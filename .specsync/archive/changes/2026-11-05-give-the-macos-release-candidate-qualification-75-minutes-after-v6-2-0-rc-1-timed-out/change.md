---
id: give-the-macos-release-candidate-qualification-75-minutes-after-v6-2-0-rc-1-timed-out
state: archived
type: operations
base_commit: 2ebe1f44a527f9b04dd73d3326dadd2bd481b055
---

# Give the macOS release candidate qualification 75 minutes after v6.2.0-rc.1 timed out

## Intent

Give the macOS release candidate qualification 75 minutes after v6.2.0-rc.1 timed out

## Affected Canonical Specs

- None

## Acceptance Criteria

- The release.yml qualify job allows 75 minutes, so a v6.2.0 candidate that passes every test on macos-15 is not cancelled during the release build; the 6.2.0 notes record rc.1's timeout.

## No-spec Rationale

Release workflow job timeout and release notes only; no runtime, canonical API, Action default, risk or provenance policy changes.
