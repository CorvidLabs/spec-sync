---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: docs
---

# Docs

`site/src/content/docs/workflow.md` gets one paragraph, in the paragraph that explains what
`change check` compares. It says three things:

- An unsuppressed warning in a module the change declares (for example an undocumented export)
  fails `change check` on its first run. Finalization already refused it.
- This does not depend on the configured `enforcement`, which still governs `specsync check`
  only.
- The failed attempt is recorded in `verification.json` with an entry naming the effective-contract
  gate, so `change status` points back to `change check`.

Two other doc changes:

- After verification, `specs/change/change.3md` records the lesson: the first check after
  materialization was the only run that recorded a pass, and the only run in which the gate did
  not ask.
- The canonical requirement and Contract text arrive through the approved delta.

The adjacent sentence in `workflow.md` (and in `cli.md:421`) claims that release/security
classification "adds validators to the same scoped evidence path". That is not true since
REQ-change-091. It is left as written here and proposed as a follow-up, together with the
hard-coded strict classification.

`CHANGELOG.md` is written by the release change. The PR body states the observable change to
first-check outcomes so the release notes can carry it.
