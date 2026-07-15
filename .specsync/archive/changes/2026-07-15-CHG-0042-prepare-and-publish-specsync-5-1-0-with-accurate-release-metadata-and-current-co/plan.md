---
change: CHG-0042-prepare-and-publish-specsync-5-1-0-with-accurate-release-metadata-and-current-co
artifact: plan
---

# Plan

1. Archive the four accepted change workspaces after their delivery commits are on
   the comparison base.
2. Bump canonical package metadata to 5.1.0 and update the explicit Trust workflow
   binary pin without rewriting historical examples or archived evidence.
3. Promote the Unreleased changelog entries into a dated 5.1.0 section with a
   release link and leave a clean Unreleased section for future work.
4. Refresh Spec Kit and OpenSpec comparisons, add a BMAD comparison, and summarize
   the honest positioning on the main comparison page.
5. Run formatting, linting, complete tests, strict specs at 100% coverage,
   documentation checks/build, release build, dependency audit, Trust, and Attest.
6. Publish a reviewable release PR; merge and tag only after hosted CI and human
   closing approval are current.
