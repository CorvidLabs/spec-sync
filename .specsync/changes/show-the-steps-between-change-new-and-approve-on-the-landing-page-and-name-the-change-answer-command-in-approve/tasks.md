---
change: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
artifact: tasks
---

# Tasks

- [x] Write the two landing-page tests and the classifier case, and see them fail at 9c28b332.
- [x] Name the `change answer` command in the two approve refusals.
- [x] Add the steps between `change new` and approve, and the archive-first closeout, to the landing page.
- [x] Route `site/src/content/docs/index.md` to the full lane in `classify-ci-paths.sh`.
- [x] Update `specs/change/change.spec.md` (Error Cases row, version, Change Log) and `specs/change/testing.md`, and add the CHANGELOG entry.
- [x] Run the documented-workflow tests, the classifier test, `fledge lanes run verify` on rust 1.89.0 and `specsync check --strict`.

Human gates, not tasks: Leif's scope approval of this change, and Leif's implementation review before `change review` and `ship`.
