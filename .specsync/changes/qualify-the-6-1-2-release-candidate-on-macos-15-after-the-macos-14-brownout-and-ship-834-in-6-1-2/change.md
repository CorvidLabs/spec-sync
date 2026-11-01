---
id: qualify-the-6-1-2-release-candidate-on-macos-15-after-the-macos-14-brownout-and-ship-834-in-6-1-2
state: draft
type: operations
base_commit: 3d3d6e6db88077b1c233b37e4958916d8da40bbd
---

# Qualify the 6.1.2 release candidate on macos-15 after the macos-14 brownout and ship #834 in 6.1.2

## Intent

Qualify the 6.1.2 release candidate on macos-15 after the macos-14 brownout and ship #834 in 6.1.2

## Affected Canonical Specs

- None

## Acceptance Criteria

- The release.yml qualify job's macOS matrix entry runs on macos-15 instead of the retired macos-14 image; its platform name, fledge asset and checksum and every other workflow line are unchanged.
- CHANGELOG.md 6.1.2 and docs/releases/6.1.2.md add #834 (#768 case 1: check --fix keeps export rows in the table that already holds documented exports; case 2 stays open), name the new candidate source commit on origin/main, and record that v6.1.2-rc.1 failed qualification because of the scheduled macos-14 brownout and is superseded by v6.1.2-rc.2.
- validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass.
- No runtime source, canonical spec, dependency, tag or release changes; v6.1.2-rc.1 is left as is.

## No-spec Rationale

Release workflow runner image and release notes only; no runtime, canonical API, Action default, risk or provenance policy changes.
