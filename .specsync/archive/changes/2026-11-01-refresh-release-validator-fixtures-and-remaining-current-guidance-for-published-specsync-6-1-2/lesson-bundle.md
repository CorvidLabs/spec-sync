# Lesson bundle: refresh-release-validator-fixtures-and-remaining-current-guidance-for-published-specsync-6-1-2

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refresh release-validator fixtures and remaining current guidance for published SpecSync 6.1.2
- **Kind**: Documentation
- **Paths**: .github/scripts/test-validate-release-candidate.py, site/src/content/docs/quickstart.md, site/src/content/examples/polyglot.mdx, site/src/content/examples/ci-gate.mdx, SECURITY.md, MIGRATION.md, site/src/content/docs/integrations/github-action.md
- **Acceptance**: In .github/scripts/test-validate-release-candidate.py, the two published-default tests that anchored on the old @main archive-gate example match the published docs: test_archive_gate_main_ref_expires_once_a_newer_release_is_published first rewrites the @v6.1.2 archive-gate example to @main and still requires the newer-release rejection, and test_postpublication_default_requires_deliberate_advance asserts the docs already pin @v6.1.2 and that the validator passes. No other test and no validator logic changes; all 66 tests pass.
- **Acceptance**: Current guidance stops describing 6.1.2 as unpublished or pinning 6.1.1: site/src/content/docs/quickstart.md (the GitHub Action candidate note), site/src/content/examples/polyglot.mdx and ci-gate.mdx (the candidate notes), SECURITY.md (the CI pinning bullet, including the main wrapper default now 6.1.2), and the github-action.md manual-CI curl download moves from v6.1.1 to v6.1.2.
- **Acceptance**: MIGRATION.md current 5.x to 6 guidance (binary pin and install tag, Homebrew line, Action pins, version check comment and the explicit Action pin paragraph) moves from 6.1.1 to 6.1.2; historical 5.0 and release-specific history text is unchanged.
- **Acceptance**: validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, change audit passes with both active changes, and the pre-push gate passes. No runtime source, canonical spec, dependency, workflow, tag or release changes.

## Evidence

- Verification commit: `3beb620f4d51b1c192ee62bb5597ac49233d91ad`
- Base commit: `94585d49d549c581db498eca25899856e27fa43c`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

The companion change `advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks` on this PR (#836) moved the archive-gate example in github-action.md from `@main` to `@v6.1.2`. Two release-validator tests anchored on the old `@main` text, so 2 of 66 failed. The fixture edit was made in the working tree to satisfy that change's validator criterion, but the test file is outside its approved paths, and `change audit` refused it as uncovered. 0xLeif chose a second small change covering the test file plus the remaining current guidance that still says 6.1.2 is unpublished or pins 6.1.1.

SpecSync 6.1.2 is published (GitHub release v6.1.2 Latest, crates.io 6.1.2; Homebrew is homebrew-tap#40, open). github-action.md is also in the companion change's paths; this change touches only its manual-CI curl line. The test-file edit is already in the working tree history (commit d1006c6f); the documentation edits are made only after approval.

## From the change's design.md

# Design

Test fixtures follow the published docs instead of the candidate-window text: the expiry test rewrites the published `@v6.1.2` archive-gate example back to `@main` before advancing the published default to 6.2.0, so it still proves the `@main` allowance ends; the post-publication test asserts the docs already pin `@v6.1.2` with `version: '6.1.2'` and that the validator passes. Validator logic is unchanged.

Documentation edits are textual version refreshes in current guidance only: remove "use 6.1.2 only after publication" notes, state the main wrapper default as published 6.1.2 in SECURITY.md, move MIGRATION.md current pins and checks to 6.1.2, and point the manual-CI curl at the v6.1.2 asset. Historical sections stay as written.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
