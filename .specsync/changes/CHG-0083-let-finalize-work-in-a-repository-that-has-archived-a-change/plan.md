---
change: CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change
artifact: plan
---

# Plan

1. Add a failing test: a directory candidate whose expansion is rejected.
2. Introduce `candidate_scope_admits`.
3. Apply it to all four guards — index, modified, visibility, fsmonitor.
4. Confirm finalize clears the guard in a repository with archived changes.
