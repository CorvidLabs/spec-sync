# Lesson bundle: qualify-the-6-1-2-release-candidate-on-macos-15-after-the-macos-14-brownout-and-ship-834-in-6-1-2

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Qualify the 6.1.2 release candidate on macos-15 after the macos-14 brownout and ship #834 in 6.1.2
- **Kind**: Operations
- **Paths**: .github/workflows/release.yml, CHANGELOG.md, docs/releases/6.1.2.md
- **Acceptance**: The release.yml qualify job's macOS matrix entry runs on macos-15 instead of the retired macos-14 image; its platform name, fledge asset and checksum and every other workflow line are unchanged.
- **Acceptance**: CHANGELOG.md 6.1.2 and docs/releases/6.1.2.md add #834 (#768 case 1: check --fix keeps export rows in the table that already holds documented exports; case 2 stays open), name the new candidate source commit on origin/main, and record that v6.1.2-rc.1 failed qualification because of the scheduled macos-14 brownout and is superseded by v6.1.2-rc.2.
- **Acceptance**: validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass.
- **Acceptance**: No runtime source, canonical spec, dependency, tag or release changes; v6.1.2-rc.1 is left as is.

## Evidence

- Verification commit: `4eaaae45eef8db745eb5e66705ea464feec35f76`
- Base commit: `3d3d6e6db88077b1c233b37e4958916d8da40bbd`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

v6.1.2-rc.1 (tag object 520f1cc0, commit 8873c08d, the #822 merge) failed qualification in release run 41340279221. The Ubuntu qualify job passed; the macOS job was cancelled before any step because GitHub failed every macos-14 job during the scheduled brownout of 2026-11-01 14:00 UTC to 2026-11-02 00:00 UTC, and the image retires on 2026-11-02 (actions/runner-images#13518). A tag push runs the workflow from the tagged commit, so rc.1 can never requalify; it is left as is.

The owner chose to fix the runner pin on main and ship #834 (#768 case 1) in 6.1.2, since main already carries it and validate requires the candidate to be an ancestor of origin/main. rc.2 is tagged on the merge of this change.

## From the change's testing.md

# Testing

Run validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py and record their exits. The hosted proof is rc.2 qualification passing on ubuntu-latest and macos-15. No green result is inferred from historical receipts.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
