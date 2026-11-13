---
change: the-persistent-read-cache-stays-bounded
artifact: research
---

# Research

- Approvals are consumed only by `admissible_archive_introductions` (comparing a change's earlier
  introductions) and `closing_ledger_extends_all_introductions`, both for one change id.
- `archive_introductions_from_listing` already reads each package's `approvals.json` with
  `git show <commit>:<dir>/approvals.json` on the cold path, so re-reading it by the same name is
  exact and needs no object id.
- The persistent cache is disabled in shallow, grafted, replaced and partial clones, so a
  `Committed` ledger's blob is always local when the cache answered.
