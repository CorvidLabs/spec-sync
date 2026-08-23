---
change: name-what-merging-before-finalize-actually-costs
artifact: context
---

# Context

#687, from measurement on a real repository under owner authorization.

## What the tool said

Four sites warned in substantially these words:

    merge only after finalize — merging first orphans verification evidence and strands the change

That prices the loss as ONE record — the reader's own, recoverable, theirs. Someone weighing
"ship now vs finalize first" reasonably concludes the cost is local.

## What it actually costs

An unfinalized change never reaches `accepted` or `archived`, so it never becomes an "accepted or
archived successor" — and every EARLIER accepted change sharing a delivery input with it can no
longer archive:

    specsync change archive CHG-0036-...
    error: delivery input `specs/algorand-hub/algorand-hub.spec.md` (owner `algorand-hub`)
    changed after acceptance and no accepted or archived successor change covers it

Measured on the affected repository: the predecessor was last updated 2026-08-21; four later
commits touched the same spec, all four still in `verifying` because each was merged before
finalize. Coupled set on that one spec: 13 changes.

So merging one change early does not strand one record. It strands that record AND blocks an
unbounded set of predecessors, with the only exits being to finalize the successors or reopen the
predecessors.

## Why the wording is the fix and not the rule

Each merge-early decision is individually small and locally recoverable. The aggregate is a
lifecycle that cannot drain. Nobody could price that decision correctly from the current text,
because the text describes a different, smaller cost.

The repository owner in question was told the documented cost twice before merging and chose to
proceed. They were told the truth as the tool states it; the tool understates it.

## What this change does NOT claim

An earlier explanation held that `finalize` cascades forward into siblings. That was wrong and was
withdrawn after an archive run showed the FIRST change failing immediately. A later prediction —
that finalizing the successors would clear the predecessor — was also wrong, refuted by a second
run. Both are recorded on #687 and #688.

This change fixes the disclosure. It makes no claim about the shape of the coupling beyond what
the measured error message states, and it does not assert that following the corrected advice
clears any existing pile.
