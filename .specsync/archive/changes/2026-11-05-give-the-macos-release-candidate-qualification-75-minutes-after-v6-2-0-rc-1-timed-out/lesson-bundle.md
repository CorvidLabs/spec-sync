# Lesson bundle: give-the-macos-release-candidate-qualification-75-minutes-after-v6-2-0-rc-1-timed-out

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Give the macOS release candidate qualification 75 minutes after v6.2.0-rc.1 timed out
- **Kind**: Operations
- **Paths**: .github/workflows/release.yml, CHANGELOG.md, docs/releases/6.2.0.md
- **Acceptance**: The release.yml qualify job allows 75 minutes, so a v6.2.0 candidate that passes every test on macos-15 is not cancelled during the release build; the 6.2.0 notes record rc.1's timeout.

## Evidence

- Verification commit: `ca1268aa3f114fea84c1e9efe5db9f91695c1e86`
- Base commit: `2ebe1f44a527f9b04dd73d3326dadd2bd481b055`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

v6.2.0-rc.1 qualification (run 41971049002) passed on ubuntu in 11 minutes. On macos-15 every test passed (2764 unit tests in about 33 minutes, 478 integration tests in about 5.5 minutes), but the job hit its 45-minute `timeout-minutes` during the release `build` task and was cancelled. macOS qualification has grown from 28 minutes (6.1.0, 6.1.1) to 39 (6.1.2) to over 45. 0xLeif chose to raise the timeout for now and address the slowness separately.

## From the change's testing.md

# Testing

Workflow-only: `python3 .github/scripts/validate-workflow-runtime-pins.py` and `test-validate-release-candidate.py` pass. The proof is the next candidate's macos-15 qualification completing.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
