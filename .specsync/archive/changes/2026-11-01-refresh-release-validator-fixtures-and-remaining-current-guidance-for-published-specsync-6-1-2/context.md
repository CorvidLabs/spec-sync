---
change: refresh-release-validator-fixtures-and-remaining-current-guidance-for-published-specsync-6-1-2
artifact: context
---

# Context

The companion change `advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks` on this PR (#836) moved the archive-gate example in github-action.md from `@main` to `@v6.1.2`. Two release-validator tests anchored on the old `@main` text, so 2 of 66 failed. The fixture edit was made in the working tree to satisfy that change's validator criterion, but the test file is outside its approved paths, and `change audit` refused it as uncovered. 0xLeif chose a second small change covering the test file plus the remaining current guidance that still says 6.1.2 is unpublished or pins 6.1.1.

SpecSync 6.1.2 is published (GitHub release v6.1.2 Latest, crates.io 6.1.2; Homebrew is homebrew-tap#40, open). github-action.md is also in the companion change's paths; this change touches only its manual-CI curl line. The test-file edit is already in the working tree history (commit d1006c6f); the documentation edits are made only after approval.
