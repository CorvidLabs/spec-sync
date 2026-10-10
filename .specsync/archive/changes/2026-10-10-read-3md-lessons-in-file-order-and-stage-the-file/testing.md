---
change: read-3md-lessons-in-file-order-and-stage-the-file
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-106 | `a_module_3md_file_shows_later_planes_as_lessons_and_leaves_context_md` passes with a higher-z context plane first, names the 3md file in the lesson bundle, and falls back to context.md when the 3md file does not parse. |

- `cargo test --bin specsync -- a_module_3md_file_shows_later_planes` passed.
