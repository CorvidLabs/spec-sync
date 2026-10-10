---
change: expect-a-module-3md-file-in-the-lifecycle-commit-scope
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-106 | `lifecycle_commit_scope_names_exactly_what_the_change_owns` passes with `specs/auth/auth.3md` in the owned set. The full `cargo test` suite passed locally. |

- `cargo test --bin specsync -- lifecycle_commit_scope_names_exactly_what_the_change_owns` passed.
- `cargo test` passed.
