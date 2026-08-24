---
change: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
artifact: research
---

# Research

`docs/6-0-findings.md` finding 10 established that lessons belong in `specs/<module>/context.md`
rather than in the change: a per-change lessons file dies with the change, and the value is that a
module accumulates what was learned about it across every change that touched it.

This change is the other half of that finding — the reading half. Writing lessons into a file that
nothing surfaces is the same failure as not writing them.

`specs/cmd_change/context.md` supplied the thin-dispatch constraint that reshaped the
implementation. `specs/change/context.md` supplied the fail-open/fail-closed distinction: evidence
validation fails closed throughout that module, so an affordance that fails open needs to say why.
