---
change: coverage-reports-unspecced-modules-in-a-stable-sorted-order
artifact: testing
---

# Testing

The unit test `coverage_reports_unspecced_modules_in_sorted_order` builds coverage over eight
configured modules (a `HashMap`). It fails when the sort is removed.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| validator coverage reporting (no requirement change) | unit `coverage_reports_unspecced_modules_in_sorted_order`; coverage/validator/generator bin tests 172/172 and coverage/generate/scaffold integration 110/110 unchanged |
