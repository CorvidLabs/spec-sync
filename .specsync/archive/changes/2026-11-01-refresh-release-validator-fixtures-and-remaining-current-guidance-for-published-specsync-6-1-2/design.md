---
change: refresh-release-validator-fixtures-and-remaining-current-guidance-for-published-specsync-6-1-2
artifact: design
---

# Design

Test fixtures follow the published docs instead of the candidate-window text: the expiry test rewrites the published `@v6.1.2` archive-gate example back to `@main` before advancing the published default to 6.2.0, so it still proves the `@main` allowance ends; the post-publication test asserts the docs already pin `@v6.1.2` with `version: '6.1.2'` and that the validator passes. Validator logic is unchanged.

Documentation edits are textual version refreshes in current guidance only: remove "use 6.1.2 only after publication" notes, state the main wrapper default as published 6.1.2 in SECURITY.md, move MIGRATION.md current pins and checks to 6.1.2, and point the manual-CI curl at the v6.1.2 asset. Historical sections stay as written.
