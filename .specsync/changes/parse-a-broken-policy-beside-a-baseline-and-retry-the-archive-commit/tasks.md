---
change: parse-a-broken-policy-beside-a-baseline-and-retry-the-archive-commit
artifact: tasks
---

# Tasks

- [x] Parse `.specsync/sdd.json` even when a workflow-v2 baseline exists.
- [x] Commit an already-archived package when finalize is run again.
- [x] Tell the operator to push the archive commit before merging.
- [x] Reject an escaping principles path instead of reporting it present.
