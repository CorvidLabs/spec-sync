---
change: ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so
artifact: design
---

# Design

`ship_next_action(push, wait, siblings_before, fold_targets, bundle) -> String`, pure.

The existing push/wait/siblings matrix is preserved exactly and becomes the *tail*. When
`fold_targets` is non-empty, the fold-back instruction is prepended:

    write lessons into <targets> from <bundle>, then <existing guidance>

Empty targets return the tail unchanged — the control case, asserted directly.

## Why pure

The regression this guards is not "the code is wrong today". It is a future edit to one verb's
guidance that forgets the other, which is exactly how the defect arose. A pure function with
tests makes the coupling structural instead of remembered.

## Why not reuse `lessons_next_action`

It ends in "then merge the PR on GitHub". Ship's tail is conditional on `--push`, `--wait`, and
sibling changes, so reuse would emit two different merge instructions in one sentence. The two
verbs share the fold-back CLAUSE, not the whole sentence.
