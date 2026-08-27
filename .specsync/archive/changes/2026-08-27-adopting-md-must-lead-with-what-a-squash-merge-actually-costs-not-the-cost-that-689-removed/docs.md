---
change: adopting-md-must-lead-with-what-a-squash-merge-actually-costs-not-the-cost-that-689-removed
artifact: docs
---

# Docs

`docs/ADOPTING.md` is the page an adopter is pointed at first, and its "Things that will bite you"
section contradicted itself about the one decision it tells them to make before adopting.

## What it said

At `:109-112`, a squash-merge *"forc[es] a full re-verify AND a fresh independent review, which is
the one step that needs a human."* At `:126-129`, seventeen lines later, *"a squash no longer forces
a re-verification."*

Both were in the same section. #689 shipped the fix; #692 added the correction as a later commit
and left the lead claim standing. So the stale claim came first, in the paragraph written to alarm,
and the retraction sat under a heading — **"Half of this is now fixed"** — that reads like a
footnote.

## What it says now

The section leads with what a squash actually costs: the **independent review**, and only that,
because the review check walks the commits between the review and `HEAD` and a squash makes that
walk impossible rather than merely false. A separate paragraph states that re-verification is no
longer among the costs, and why — ship readiness asks whether the recorded plan and tree still
match what was verified, which holds under every merge strategy.

The heading framing changed from the maintainer's question (*what did we fix*) to the adopter's
(*what does this cost me*). That framing is what put the stale claim first: "half of this is now
fixed" is only meaningful to someone who knew the other half.

## The statistic

The same paragraph read: *"89% of its own archived changes have an unreachable verification commit
— 19 of 172."* Read plainly, 19 of 172 is 11%, not 89%. The 89% was correct; the parenthetical was
the **reachable** count presented where a reader expects the unreachable one.

Re-measured on `main`, walking every archived change's recorded verification commit against
`git merge-base --is-ancestor … HEAD`:

    archived changes:  198
    reachable:          21   (10.6%)
    unreachable:       177
    no commit recorded:  0

Now stated in one direction only — "only 21 of its 198 archived changes still have a reachable
verification commit" — so there is nothing for the reader to reconcile.

`CorvidLabs/spec-sync#694` is still open and still the right pointer for the review question; the
reference is retained and rewritten to the full `CorvidLabs/spec-sync#694` form used elsewhere in
the file.
