---
change: CHG-0160-succession-must-be-ordered-by-when-a-change-happened-not-by-how-it-is-named
artifact: context
---

# Context

Step 4 of the change-identity work, which retires the `CHG-NNNN` ordinal in favour of a slug.

Succession is the one place the ordinal was doing real work rather than decorative work: it
supplied a total temporal order derived from the ID alone, with no I/O and stable across clones.
Removing it naively would have been the worst kind of regression — every comparison would still
compile, still run, and quietly mean "alphabetical" instead of "later".

The investigation that preceded this change called it the single hardest thing about the
redesign. Reading the call sites closely made it smaller than it looked: the two checks
surrounding the happens-before guard already establish the property, the records are already
loaded everywhere chronology is actually needed, and three of the six sites wanted nothing more
than a deterministic order for serialization.

It also turned out the ordinal was not merely unnecessary here but actively wrong: two sorts
over the same list disagree at five digits, and one of them feeds `scope_digest`.
