# Lesson bundle: merge-driver-keeps-a-one-sided-requirement-reorder

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Merge driver keeps a one-sided requirement reorder
- **Kind**: BugFix
- **Specs**: merge
- **Paths**: src/merge_driver.rs
- **Acceptance**: a one-sided reorder of existing requirement blocks survives the merge in either direction; divergent reorders conflict

## Evidence

- Verification commit: `fbdd2f484998fcc37cc4c7d70c2b699928a65d53`
- Base commit: `949ced6a4935688757ed82ea74819fdc6494691b`
- Verified by: `specsync check --spec merge`

## From the change's context.md

# Context

#874 + t384 on #1058: merge_requirements seeded output order from the ancestor, silently dropping a one-sided reorder; and a reorder beside a deletion was masked by comparing full surviving sets. Order is compared over ids surviving on both sides; one-sided reorder wins, divergent reorders conflict, deletions keep their per-block rules. Cites REQ-merge-006 (clean only when the result keeps everything both sides wrote); no new requirement text (t370).

## From the change's testing.md

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

## Where these lessons go

- `specs/merge/merge.3md`
