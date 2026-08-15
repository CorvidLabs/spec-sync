---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: research
---

# Research

Two choke points, not one: `exports/mod.rs`'s read for source, and
`validator.rs`'s read for the spec body. They share no call frame.
`specsync issues` is a third path that bypasses both.

The false-positive risk was measured before designing, not assumed: twelve
triples, three files, two of them scanned on every run. That measurement is what
ruled out both obvious designs and forced the two-signal composition.

Recorded because it nearly shipped: this fix arrived with `conflicted_union`
short-circuited by `if true { return None; }` and `document_conflict_hunks`
returning `Vec::new()`, so both halves were dead inside working machinery. The
acceptance test "does not fire on our own repo" PASSED against that, because a
disabled guard fires on nothing. Only running the repro alongside it exposed the
no-op. An acceptance test that asserts an absence must be paired with one that
asserts a presence.
