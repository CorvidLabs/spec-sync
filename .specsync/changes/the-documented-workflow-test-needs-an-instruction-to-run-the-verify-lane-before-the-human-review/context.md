---
change: the-documented-workflow-test-needs-an-instruction-to-run-the-verify-lane-before-the-human-review
artifact: context
---

# Context

Issue #959, from the Codex review of #952. The documented-workflow harness ran the fixture's
verify lane, and marked the tip verified, at any prose line containing "verify lane", so
`# skip the verify lane` would pass. And it checked the order only when `specsync change review`
ran, so moving `# after the human implementation review` above the verify-lane line would pass
too. Both are ways the test could pass on a page that no longer teaches verify-then-review.

The fix: a prose step runs the lane only when it tells the reader to (`run`/`rerun` ... verify
lane), and the human-implementation-review prose is an ordering event that needs the lane run on
the current tip. Two mutation tests run the README block through the harness with the reworded
line and with the reordered block, and require both to be refused.
