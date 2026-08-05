---
change: CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change
artifact: design
---

# Design

`candidate_scope_admits(candidates, path)` replaces exact membership at each
guard. A path is in scope when it equals a candidate, or when a candidate is one
of its ancestor directories.

The comparison tests the byte at the separator boundary, so `a/bc` cannot admit
`a/b`.

This is not a relaxation. A directory candidate requests everything beneath it;
admitting those files states the scope that was already asked for. Rejecting
them was the defect.

Rejected: filtering directories out of the candidate set upstream. The producers
(`acceptance_manifest_internal`, the workspace-path caller) pass archived-change
directories deliberately. Removing them would silently narrow what finalize
hashes rather than fix the guard that misreads it.
