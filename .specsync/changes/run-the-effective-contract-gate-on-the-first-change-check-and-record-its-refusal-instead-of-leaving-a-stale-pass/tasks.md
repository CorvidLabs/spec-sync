---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: tasks
---

# Tasks

- [x] Add the discriminator and control tests in `testing.md` and record which fail on origin/main f240d37.
- [x] Attribute effective-contract errors to modules privately, preserving every existing caller's output.
- [x] Have `verify_change_locked` run the gate with the verifying change included on its first check.
- [x] Record a declared-module gate refusal as a failed attempt with the named gate entry, after the unchanged tasks and evidence checks.
- [x] Keep non-attributable gate failures as pre-attempt refusals.
- [x] Document the first-check behavior in `site/src/content/docs/workflow.md`.
- [x] Update the change testing, tasks and `change.3md` lesson after verification.
- [x] Run the regression lanes in `testing.md` and record the observed results.

## Human gates

Scope approval of this change by a human is required before implementation starts. No approval
is recorded by this draft.

Implementation review (`specsync change review <id> --reviewer <human>`) by a human follows a
current `change check`. Finalization, merge and any release are separate human decisions.

A separate human decision is proposed, not part of this scope: whether to remove the hard-coded
SpecSync paths and modules from `change_requires_strict_validation` and correct the docs claim
that the classification adds validators.

## After verification

Run `fledge lanes run pre-push` before every push and `fledge trust verify` before calling the
work complete. Hosted CI and Trust results for the exact head are recorded as evidence, not
assumed.
