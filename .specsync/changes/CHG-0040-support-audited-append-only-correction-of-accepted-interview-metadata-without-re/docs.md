---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: docs
---

# Docs

- Add `change correct` syntax, supported fields, required audit inputs, JSON shape, and failure cases
  to `site/src/content/docs/cli.md`.
- Extend `site/src/content/docs/workflow.md` with a focused accepted-definition correction workflow:
  correct, complete any newly selected artifacts, approve, verify, and accept.
- Explicitly contrast `change correct` with `change reopen`: correction changes the effective
  historical classification; reopen only refreshes stale delivery evidence.
- Explain that original answers and prior evidence remain visible, artifacts are monotonic, and
  already-applied semantic deltas never replay.
- Add an unreleased 5.1 changelog entry with the supported-field boundary and migration-free legacy
  behavior.
