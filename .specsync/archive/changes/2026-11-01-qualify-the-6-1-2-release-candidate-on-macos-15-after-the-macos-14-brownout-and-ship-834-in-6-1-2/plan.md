---
change: qualify-the-6-1-2-release-candidate-on-macos-15-after-the-macos-14-brownout-and-ship-834-in-6-1-2
artifact: plan
---

# Plan

## Approach

Change only the qualify job's macOS matrix runner from macos-14 to macos-15, and update the 6.1.2 CHANGELOG section and release handoff.

## Out of scope

No runtime, dependency, spec, Action default or other workflow logic changes. No tag, release, publication or floating-ref movement. rc.1 is not touched.

## Steps

Edit release.yml line for the macOS qualify runner; add #834 to CHANGELOG 6.1.2 and docs/releases/6.1.2.md with the new source and the rc.1 supersession; run the three release validators; verify, review and finalize natively before merge; then tag v6.1.2-rc.2 on the merge commit.

## Risks

macos-15 must provide Rosetta for the pinned fledge-macos-x86_64 asset, as macos-14 arm64 did; rc.2 qualification proves it. macos-15 is pinned explicitly rather than macos-latest so the qualification image does not move underneath a candidate.

## Constraints consulted

Repository AGENTS.md, docs/RELEASING.md and docs/releases/6.1.2.md; RC tags are immutable, so a bad candidate is superseded by the next N.
