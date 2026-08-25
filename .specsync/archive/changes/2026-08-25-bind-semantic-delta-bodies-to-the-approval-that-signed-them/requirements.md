---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: requirements
---

# Requirements

1. `approve` records, on the definition approval event, a digest over the exact bytes of every
   semantic delta file the change owns, keyed by module.
2. Materialization and acceptance verify those digests before any delta rewrites a canonical spec,
   and refuse with a message naming each module whose body changed after approval.
3. An approval carrying no such digest proceeds. Absent evidence is unknown, never a violation, so
   no historical archive fails on evidence that could not have been written when it was approved.
4. The new field changes no existing digest and leaves every previously written `approvals.json`
   readable and byte-identical when re-serialized.

The mapping into the canonical spec is `REQ-change-089`.
