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
  dirty tree) and calls `classify_paths_with(paths, archive_id, package_exists)`. `archive_id`
  reads `.specsync/archive/changes/<dir>/state.json`. One change's package paths count as archive
  work only on CI's terms (`classify-ci-paths.sh`): they sit beside archive paths whose
  `state.json` all name that change, the tip deleted each of them (a rename into the archive
  deletes its source), and `.specsync/changes/<id>` no longer exists. A package the tip modified,
  or one still present, is product (review of #1093). The test-only `classify_paths` keeps the
  old behaviour, with no identity.
