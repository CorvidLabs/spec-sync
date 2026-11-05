---
change: make-the-unowned-path-ownership-hint-deterministic-and-fall-back-at-depth-zero
artifact: context
---

# Context

Leif approved #841 on GitHub (review 5771062363, 2026-11-04 08:33 MST) with optional nits, and at 09:25 MST said: "Yes, merge #841 and open the nits follow-up PR." #841 merged as f65dbed. The nits are captured verbatim in hi as CHANGE-10.a, CHANGE-10.b and CHANGE-10.c.

Ownership does not change (REQ-change-053). `ownership_hint` only words the refusal after it has been decided.

Beyond the nits, this branch also carries two refinements from the SpecSync bot's review of #841 that Leif has not yet confirmed and should accept or reject in review: an undeclared module whose `[modules."<name>"] owns` covers the path is named, and equally close declared specs are all named instead of the first in `affected_specs` order.
