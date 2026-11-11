---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: plan
---

# Plan

## Approach

Keep unknown members on every in-place rewrite of a lifecycle record or ledger, through one helper.

## Out of scope

#434 parts 2 and 3 (schema_version gating, file paths in type errors); `sdd.json` policy writes;
the already-released 6.2.0, which no code here can change.

## Steps

1. Span parser, merge and splice in `src/change/unknown_members.rs`.
2. Route every in-place rewrite through it.
3. Unit tests with an older and a newer record type; an end-to-end CLI test.
4. REQ-change-079 delta, invariant 52, CHANGELOG.

## Risks

A carried member could land on the wrong record; array entries are matched only when their known
text is unchanged. A carried member could change what this binary reads; the output is re-parsed
and compared, and the write is refused otherwise.

## Constraints consulted

REQ-change-079 and the CHG-0158 forward-compatibility valve; hi CHANGE-12.c and CHANGE-13.h.
