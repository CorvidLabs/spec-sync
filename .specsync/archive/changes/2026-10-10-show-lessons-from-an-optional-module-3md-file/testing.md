---
change: show-lessons-from-an-optional-module-3md-file
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-106 | `a_module_3md_file_shows_later_planes_as_lessons_and_leaves_context_md` passes, and the existing context.md briefing tests still pass. |

- `cargo test --bin specsync -- a_module_3md_file_shows_later_planes` passed.
