# Lesson bundle: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The quickstart and two example pages read as published for 6.4.2
- **Kind**: Documentation
- **Paths**: site/src/content/docs/quickstart.md, site/src/content/examples/ci-gate.mdx, site/src/content/examples/polyglot.mdx
- **Acceptance**: site/src/content/docs/quickstart.md and site/src/content/examples/polyglot.mdx lose the sentence 'Use these pins after the 6.4.2 assets are published. Until then, pin @v6.4.1 and version: 6.4.1.' In site/src/content/examples/ci-gate.mdx the sentence 'After the 6.4.2 assets are published, pin both the Action ref and the binary. Until then, pass the explicit 6.4.1 ref and binary pins.' becomes 'Pin both the Action ref and the binary.', as it read before the 6.4.2 preparation. Nothing else on those pages changes, and no page under site/ or docs/ still says 'until then'.

## Evidence

- Verification commit: `bcb852fccd11478c30862466378f17a218218753`
- Base commit: `9caed41498939ad7fdf6d74c2dd1c4eab3413bf0`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

- The 6.4.2 preparation (#1156) added a sentence to three site pages telling readers to pin 6.4.1 until the 6.4.2 assets were published. 6.4.2 is published, so the sentences are stale.
- The change that advances the Action default named its files, and these three pages were not among them. They were found after its scope was approved, so they are their own change in the same pull request.
- Leif was shown the three edits in a table in the release-lead chat on 2026-11-24, and told a yes would stand as both the scope approval and the review of this change.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
