---
change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
artifact: context
---

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
