# Lesson bundle: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only
- **Kind**: BugFix
- **Specs**: cmd_change, change
- **Paths**: src/commands/change.rs, src/change/consider_accepted_evidence_anchor.rs
- **Acceptance**: on an archived change that finalize archived and whose archive authenticates, ship-status reports review_currency current with a finalize reason, a done review stage and no change review suggestion; an archived change that finalize did not archive gets a done review stage and no change review suggestion; an archive commit that also removes the archived change's own package files is archive_only, and another change's package beside it is product

## Evidence

- Verification commit: `3aadebb018537d9460691136ad7d1551bf55e412`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

#1038, seen in CorvidLabs/algochat#89 by the algochat worker. Leif classed it a bug fix on
2026-11-13 (coordinator chat, 04:39Z, relayed by claude-code-t370): "For an archived change,
report the review as done by finalize and suggest nothing."

- Review: `recorded_scoped_review_currency` walks every commit after the review and allows only
  files under `.specsync/changes/<id>/`. The archive commit moves the package into
  `.specsync/archive/changes/`, so on an archived change the walk always reads "stale", and the
  review stage told the reader to re-record a review that `change review` refuses for an archived
  change. Finalize already required a current review, and an archive whose terminal evidence
  authenticates still holds it. So ship-status now reports `review_currency: current`, with the
  reason "finalize recorded this scoped review when it archived the change", and a done review
  stage. The walk itself is unchanged, because REQ-change-147 still has `change status` report
  `scoped_review_current: false` for a committed archive. A corrupt archive keeps the old answer.
- Tip class: `classify_paths` counted the package files a re-archive removes from
  `.specsync/changes/<id>/` as product. Git pairs no rename for them when the archive package
  already exists, as after a reopen. CI's `classify-ci-paths.sh` counts them as archive work for
  the archive whose `state.json` names that id, and ship-status now does the same. Another
  change's package in the same commit is still product.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).

## From the change's design.md

# Design

- `change::archived_review_is_settled_by_finalize(root, record)` (`pub(crate)`) is true for an
  archived workflow-v2 record whose terminal evidence is `authenticated-history` or
  `historical-transition-unproven` and whose archive still holds the finalization record that binds
  a passing review to the accepted verification (`validate_finalization_evidence`, the check
  `finalize` makes before it resumes). A workflow-v1 archive is closed by `accept` and `archive`,
  so it is never settled by finalize, even with a `review.json` in it (review of #1093).
- `ship_status_report` uses it when a review is present. `review_currency` is then `current`,
  `review_currency_reason` says finalize recorded the review, and the review stage reads "done by
  finalize". Readiness is untouched: `ready_to_finalize` still needs a verifying change.
- Any other archived change gets a done review stage that says there is no scoped review to
  record. It never names `change review`, which refuses an archived change. Its
  `review_currency` and reason stay what the review walk reports.
- `classify_head_tip` reads each path's status (`git diff --name-status`, and `git status` for the
  dirty tree) and calls `classify_paths_with(paths, archive_identity, package_exists)`.
  `archive_identity` reads the id and workflow version from
  `.specsync/archive/changes/<dir>/state.json`. One change's package paths count as archive work
  only when the tip has the shape `classify-ci-paths.sh` accepts in its name-status mode: every
  archive path is an addition or a rename target, in one archive directory whose `state.json`
  names that change at workflow v2; every package path the tip lists is a deletion; and
  `.specsync/changes/<id>` no longer exists. A package the tip modified, one still present, or a
  file modified inside an archive that already existed, leaves the package paths product (review of
  #1093). A tip that lists no package path is judged as main judged it, and the test-only
  `classify_paths` keeps the old behaviour, with no identity.
- The real tip of a change archived after a reopen is renames into the archive, with
  `finalization.json` and `verification.json` added there and deleted from the package. `change
  reopen` moves the package out of the archive, so no archive directory sits beside it.
- Hosted CI feeds the script a name-only diff of the whole pull request, where the script never
  grants archive-only, so every archived tip gets the full lane there. The parity here is with
  the script's name-status rule.

## From the change's testing.md

# Testing

| Behaviour | Evidence |
|---|---|
| An archived change's review is done by finalize, with no re-review | `commands::change::tests::an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review` (red: `stale`; green after) |
| A re-archive tip is archive_only; another change's package is product | `commands::change::tests::an_archive_tip_that_removes_its_own_package_files_is_archive_only` (red: `product`; green after) |
| A package the tip modified, or one still present, is product, as in CI | `commands::change::tests::an_archive_tip_whose_package_is_still_present_is_product` (red: `archive_only`; green after) |
| Deleted package files beside a modified archive file are product | `commands::change::tests::deleted_package_files_beside_a_modified_archive_file_are_product` (red: `archive_only`; green after) |
| Every status that grants or withholds the exemption | `commands::change::tests::only_an_exact_archive_move_makes_package_paths_archive_work` |
| One tip gives one answer from CI's script and from ship-status | `commands::change::tests::the_own_package_exemption_agrees_with_the_ci_classifier_script` (red on the modified archive file; needs `bash` and `jq`, and says so where they are missing) |
| A workflow-v1 archive is not done by finalize and is told no review | `commands::change::tests::a_workflow_v1_archive_is_not_done_by_finalize_and_suggests_no_review` (red: `current`; green after) |
| An archived change lists no blocker, so ship-status exits 0 on it | the two archived-change tests above assert an empty `blockers` list |
| Existing tip and review semantics | `classify_paths_distinguishes_product_review_and_archive_tips` and the other ship-status tests |

Locally: `cargo test --bin specsync commands::change::`; hosted CI runs the full suite at the archived head.

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
- `specs/change/change.3md`
