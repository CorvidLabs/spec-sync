---
change: split-dependency-graph-import-analysis-and-reporting-into-focused-private-modules
artifact: tasks
---

# Tasks

- [x] Preserve baseline source and 43-test inventory.
- [x] Extract facade and graph/import/report owners with all files below 1000 lines.
- [x] Relocate all tests and update canonical topology documentation.
- [x] Prove exact body/literal preservation and obtain independent engineering review.
- [x] Run the available authorized source, focused test, lint, build and strict/pre-push checks.

## Remaining lifecycle gates

The real scoped lifecycle verification commit is produced by `change check --commit`; that command must succeed before this local checkpoint is reported. Human implementation review and normal CI/trust remain required before shipping; neither has been performed or recorded by the agent.
