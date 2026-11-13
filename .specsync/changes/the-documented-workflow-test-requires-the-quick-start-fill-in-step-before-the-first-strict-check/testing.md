---
change: the-documented-workflow-test-requires-the-quick-start-fill-in-step-before-the-first-strict-check
artifact: testing
---

# Testing

- Integration `the_quick_start_fills_in_the_scaffold_before_the_first_strict_check` in
  `tests/integration/documented_workflows.rs`: the page as written passes; the same page with the
  fill-in line moved below `specsync check --strict` is refused (fails on main: the mutation is
  accepted).
- `landing_page_change_workflow_runs_from_its_quick_start` and the other documented-workflow
  tests still pass.
