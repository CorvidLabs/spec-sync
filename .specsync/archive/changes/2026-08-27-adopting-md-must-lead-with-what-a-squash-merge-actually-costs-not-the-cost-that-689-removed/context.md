---
change: adopting-md-must-lead-with-what-a-squash-merge-actually-costs-not-the-cost-that-689-removed
artifact: context
---

# Context

## What led here

Found by an agent backfilling the 6.0 changelog (#731), reading #692's shipped diff rather than its
title. Filed as #729.

The mechanism is worth recording because it is not carelessness. #692 landed the merge-strategy
warning; a later commit **in the same PR** corrected the cost downward after #689 shipped. Adding
the correction is the natural edit; going back to weaken the alarming paragraph you wrote three
commits ago is not. The result was a document that was accurate in sum and misleading in order.

## What a session picking this up needs to know

**The remaining cost is real and must not be softened away.** A squash genuinely does break the
independent review: that check walks the commits between the review and `HEAD`, and a squash makes
the walk *impossible* rather than merely false. #694 is open and needs a decision about what a
review proves, not a patch. Removing the warning entirely would be the opposite error to the one
being fixed here.

**Both numbers were verified, not copied.** The repository is still squash-only
(`{"merge":false,"rebase":false,"squash":true}`), and reachability was re-measured across all 198
archived changes rather than trusting the figure already in the file — which is how the garbled
"19 of 172" was caught. The 89% turned out to be *correct*; only its presentation was wrong. The
instinct on seeing "89% … 19 of 172" is to call the percentage the error, and that instinct is
wrong here.

## Ruled out

**Deleting the "Half of this is now fixed" paragraph rather than reframing it.** The information in
it is what an adopter most needs — that re-verification is no longer a cost — and it is the half a
reader is most likely to have heard the old version of from someone else. It is kept, stated
positively, and moved above the settings check so it cannot be missed.
