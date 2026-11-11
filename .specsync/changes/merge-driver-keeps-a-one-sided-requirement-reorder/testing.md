---
change: merge-driver-keeps-a-one-sided-requirement-reorder
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-merge-006 | merge_driver unit tests |

## How the merge_driver tests run

`change check` executes only the in-process spec↔code sync (recorded as
`specsync check --spec merge`); `.specsync/sdd.json` verification commands are
not executed by the lifecycle (#893 made that explicit). The merge_driver unit
tests therefore run two other ways:

- Locally at the final head: `cargo test merge_driver` — 30 passed, 0 failed,
  including `a_one_sided_requirement_reorder_is_kept`,
  `a_reorder_plus_a_deletion_keeps_the_reorder`, and
  `divergent_requirement_reorders_conflict` (which asserts the merged content
  contains `<<<<<<<` markers, not just `!clean`).
- In hosted CI: the `test` job runs the full suite at the final head.

