---
change: cover-the-integration-fixtures-the-ordinal-retirement-rewrote
artifact: context
---

# Context

Scope follow-on to `retire-the-ordinal-and-keep-the-ledger-readable-forever`.

That change declared `src/change.rs` and `src/change_tests.rs`. The work also had to rewrite
integration fixtures that hard-coded the `CHG-NNNN` identity shape — `tests/integration/change.rs`
and `tests/integration/comment.rs` — and those two paths were not declared. Scope freezes at
approval, so the archived change cannot be widened; this covers them instead.

Worth recording rather than quietly fixing: the omission was mine, and the tool caught it after
the fact rather than before, because the fixtures only became visible as in-scope when the patch
was applied. That is the same shape as #542 — blast radius is knowable at compile and test time,
not at the interview.
