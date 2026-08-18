---
change: CHG-0143-the-sequence-ledger-gate-must-judge-a-branch-by-its-own-history-not-by-origin
artifact: context
---

# Context

`adbfb442` fixed #533: a stale working-tree ledger was being staged over a higher
committed mark, so SpecSync's own materialize commit rewrote the high-water mark
downwards. The write-side floor that fixed it is correct and is not in question.

The same change added a read-side gate to `validate_change_sequences`, and that
gate asked the wrong question. It compared the working-tree ledger against
`origin/main`:

    if let Some(remote) = remote_sequence_high_water(root)
        && remote > ledger.sequence

Every unrebased branch trips that. A branch cut before the default branch
advanced holds an older ledger which is perfectly consistent with its own
history, and the gate refused it:

    $ specsync change new "work on a behind branch"
    error: change sequence ledger claims CHG-0001 but the default branch has
    already recorded CHG-0002; restore it with `git checkout origin/HEAD --
    .specsync/change-sequence.json` before continuing
    rc=1

Nothing is wrong with that branch. The message diagnoses it as corruption and
prescribes a recovery that is not needed. `check` emitted the same text as a
warning on every run.

The gate also prevented nothing. #523 already floors ALLOCATION against the same
remote mark, so a behind branch cannot remint an ordinal: with the gate removed,
`change new` on that branch allocates CHG-0003, not a colliding CHG-0002.

## Why no drill caught it

The 55-drill board stayed green through the regression. Drill 051 is the #533
pin and it passed in both the broken and the fixed state, because it only
exercised the WRITE path — `check --commit` refusing to lower a committed mark.
No drill built a branch that was behind origin, so the read path had no coverage
at all. That gap is closed in the same work (sandbox #86).

## What the gate is actually for

A ledger that went backwards relative to what THIS branch already recorded. That
is a fact about the branch, not about its distance from origin — and on disk the
two are indistinguishable, which is why comparing against origin cannot separate
them.
