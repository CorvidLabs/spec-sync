---
change: delta-materializer-matches-and-keeps-titled-requirement-headers
artifact: testing
---

# Testing

change_tests/titled_requirement_header_delta.rs: eleven tests covering every
header form and boundary from #1039's three review rounds.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-152 | change_tests titled-header tests |
| REQ-validator-047 | same suite through the shared parser |

## How the titled-header tests run

`change check` executes only the in-process spec↔code sync (recorded as
`specsync check --spec change --spec validator`); `.specsync/sdd.json`
verification commands are not executed by the lifecycle. The titled-header
suite therefore runs two other ways, and both run again at the post-approval
re-verify:

- Locally at the final head: `cargo test titled_requirement_header`.
- In hosted CI: the `test` job runs the full suite at the final head.
