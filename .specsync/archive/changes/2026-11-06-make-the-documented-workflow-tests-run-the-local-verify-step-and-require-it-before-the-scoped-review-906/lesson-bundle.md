# Lesson bundle: make-the-documented-workflow-tests-run-the-local-verify-step-and-require-it-before-the-scoped-review-906

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make the documented workflow tests run the local verify step and require it before the scoped review (#906)
- **Kind**: Documentation
- **Paths**: tests/integration/documented_workflows.rs, site/src/content/docs/index.md, CHANGELOG.md, .chough/
- **Acceptance**: In the README and landing-page workflow tests, a prose step naming the verify lane runs the fixture lane (specsync check --strict), which must pass, on the current tip; change review is allowed only after that ran on the tip it reviews (DOCS-2, #906).
- **Acceptance**: The tests fail if a documented closeout drops its verify step or puts it before change check --commit or after change review: the README with its verify line removed fails at the review, and the landing page fails at 3758d970 because it has none.
- **Acceptance**: The landing page closeout gains the verify step after change check --commit, worded like the README (run the local verify lane; CI does not test a tip whose change is still open), and its test passes.
- **Acceptance**: The landing page's Quick Start tells the reader to fill in the add-spec auth scaffold before the change starts, in one line after add-spec: '# fill in specs/auth/ (the spec and its companions; Spec Format guide)'. The landing-page tests fill it in at that line as a reader would (real sections, status: active, login documented, no scaffold markers), and the fixture lane stays specsync check --strict (Leif, 2026-11-06, option A).
- **Acceptance**: No assertion of the existing documented-workflow tests is loosened.

## Evidence

- Verification commit: `990de72b4e897bfdb651a56cf37dfe3a833df5e4`
- Base commit: `0b209a5c1398b9e9b881ced48a3145b17e8492b9`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Issue #906 comes from the Codex review of #898 (r4489185060). The README test ran the README's `# run the local verify lane` line as prose with no action. `change review` then succeeded anyway, because SpecSync never runs or observes project commands. So the test passed if the step was dropped or moved after the review. The landing page has no verify step at all. DOCS-2 (Leif, 2026-11-05 20:22 MST, relayed by orc t18) says the closeout is verify, the implementation review, then archive, before the push.

The fixture has no product test suite, so its verify lane is `specsync check --strict`. That is the real check a reader's lane would include for this project. SpecSync's product behavior does not change: `change check` still runs no project commands.

The branch is stacked on #917 (0b209a5c), which is stacked on #912. All three edit `tests/integration/documented_workflows.rs`. main is merged in as the earlier PRs land. The no-scope approve probe on this branch still fails until #917's approved implementation lands; that failure is #917's, not this change's.

Re-scoped on 2026-11-06. Built as first approved (a6a028ee), the README passed and the landing page's new verify step failed. The fixture's `specsync check --strict` refused the `add-spec auth` scaffold: status draft, scaffold markers in the spec and its four companions, and draft-only sections, 12 warnings in all. The page never told the reader to fill the scaffold in, so a reader whose lane is strict (the same Quick Start lists `specsync check --strict`) would hit the same red result. Leif chose option A: one more page line after `add-spec auth`, the harness fills the scaffold in there, and the lane stays strict. Rejected: a non-strict lane for this fixture (weaker, and still red for a strict reader), and a README-only test now (leaves DOCS-2 unmet on the landing page). Finding and options: /Users/leif/Development/_CorvidLabs/_work/s5-906-strict-finding.md. The orc recorded the send-back as 0xLeif.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
