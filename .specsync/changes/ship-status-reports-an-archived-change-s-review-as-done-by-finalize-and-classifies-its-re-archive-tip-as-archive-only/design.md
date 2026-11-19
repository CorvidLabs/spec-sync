---
change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
artifact: design
---

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
