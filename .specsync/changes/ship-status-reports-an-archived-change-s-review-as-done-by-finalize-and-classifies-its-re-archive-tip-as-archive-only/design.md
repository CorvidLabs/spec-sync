---
change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
artifact: design
---

# Design

- `change::archived_review_is_settled_by_finalize(root, record)` (`pub(crate)`) is true for an
  archived record whose terminal evidence is `authenticated-history` or
  `historical-transition-unproven`. It reads the terminal-evidence verdict the read scope already
  memoizes.
- `ship_status_report` uses it when a review is present. `review_currency` is then `current`,
  `review_currency_reason` says finalize recorded the review, and the review stage reads "done by
  finalize". Readiness is untouched: `ready_to_finalize` still needs a verifying change.
- `classify_head_tip` calls `classify_paths_with(paths, archive_id)`, where `archive_id` reads
  `.specsync/archive/changes/<dir>/state.json`. When one change's package paths sit beside archive
  paths whose `state.json` all name that change, those package paths count as archive work. The
  test-only `classify_paths` keeps the old behaviour, with no identity.
