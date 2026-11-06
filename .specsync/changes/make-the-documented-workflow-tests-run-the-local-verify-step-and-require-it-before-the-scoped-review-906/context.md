---
change: make-the-documented-workflow-tests-run-the-local-verify-step-and-require-it-before-the-scoped-review-906
artifact: context
---

# Context

Issue #906 comes from the Codex review of #898 (r4489185060). The README test ran the README's `# run the local verify lane` line as prose with no action. `change review` then succeeded anyway, because SpecSync never runs or observes project commands. So the test passed if the step was dropped or moved after the review. The landing page has no verify step at all. DOCS-2 (Leif, 2026-11-05 20:22 MST, relayed by orc t18) says the closeout is verify, the implementation review, then archive, before the push.

The fixture has no product test suite, so its verify lane is `specsync check --strict`. That is the real check a reader's lane would include for this project. SpecSync's product behavior does not change: `change check` still runs no project commands.

The branch is stacked on #917 (0b209a5c), which is stacked on #912. All three edit `tests/integration/documented_workflows.rs`. main is merged in as the earlier PRs land. The no-scope approve probe on this branch still fails until #917's approved implementation lands; that failure is #917's, not this change's.
