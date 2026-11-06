---
change: teach-archive-first-closeout-in-the-readme-and-run-its-workflow-example-as-a-test-881-d5
artifact: tasks
---

# Tasks

- [x] Capture DOCS-1 to DOCS-3.a in `hi/docs.md` as Leif approved them.
- [x] Write `readme_change_workflow_archives_before_it_tells_the_reader_to_wait_for_ci` and see it fail on README.md at ad50bdef.
- [x] Reorder the README closeout to archive-first and add Chough's wording under the block.
- [x] Add the CHANGELOG entry.
- [x] Run the new test, `fledge lanes run verify` and `specsync check --strict`.

Human gates, not tasks: Leif's scope approval of this change, and Leif's implementation review before `change review` and `finalize`.
