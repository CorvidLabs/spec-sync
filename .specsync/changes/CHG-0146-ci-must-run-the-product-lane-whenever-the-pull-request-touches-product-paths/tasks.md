---
change: CHG-0146-ci-must-run-the-product-lane-whenever-the-pull-request-touches-product-paths
artifact: tasks
---

# Tasks

- [x] Reproduce on a real pull request: #629, nine source files, product lane skipped, aggregate green.
- [x] Confirm no CI run existed for ANY commit on that branch, not merely the tip.
- [x] Read the classifier flow and find that the whole-PR answer was not merely
      overridden but never computed.
- [x] Extract the decision out of inline workflow YAML into a script that can be tested.
- [x] Add tests for narrow-not-contradict, genuine narrowing, and no candidate.
- [x] Verify the tests fail against the previous unconditional override.
- [x] Run the existing classify harness to confirm no regression.
