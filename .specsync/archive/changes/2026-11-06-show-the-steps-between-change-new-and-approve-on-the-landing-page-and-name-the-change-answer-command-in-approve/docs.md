---
change: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
artifact: docs
---

# Docs

- `site/src/content/docs/index.md`, Quick Start: the interview answer and the artifacts/delta step go between `change new` and `change approve`, then the closeout, archive-first (see design).
- `specs/change/`: REQ-change-133 is materialized from the delta into `requirements.md`. `change.spec.md`'s Error Cases row for a missing acceptance criterion says the refusal names the `change answer` command, with a version bump and a Change Log row. `testing.md` names the two tests.
- CHANGELOG.md: an Unreleased / Fixed entry.
