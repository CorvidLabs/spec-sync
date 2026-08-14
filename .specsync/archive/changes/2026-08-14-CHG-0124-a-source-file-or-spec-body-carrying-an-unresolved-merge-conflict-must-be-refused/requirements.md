---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: requirements
---

# Requirements

`REQ-merge-00N` — conflict detection SHALL be available as a pure content
predicate and as a git-status query, and the git query SHALL distinguish
"unknown" from "no unmerged paths".

`REQ-exports-00N` — extraction SHALL report a scan that unioned both sides of a
conflict hunk as conflicted, carrying which side contributed which symbols,
rather than returning the union as if it were a symbol list.

`REQ-validator-00N` — a spec whose mapped source is conflicted, or whose own
body carries a conflict outside fenced code, SHALL fail validation naming the
conflict, and SHALL NOT compare the spec against the union.

Out of scope: resolving conflicts, and `merge`'s existing behaviour.
