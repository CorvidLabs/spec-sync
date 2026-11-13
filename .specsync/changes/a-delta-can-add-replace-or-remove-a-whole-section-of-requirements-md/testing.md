---
change: a-delta-can-add-replace-or-remove-a-whole-section-of-requirements-md
artifact: testing
---

# Testing

- Unit tests in `src/change_tests/requirements_section_delta.rs` (fail on main: the heading is
  refused).
- The existing delta, materializer, stale-MODIFIED and send-back tests still pass.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-162 | `src/change_tests/requirements_section_delta.rs` |
