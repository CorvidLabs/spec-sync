---
change: CHG-0086-return-src-commands-change-rs-to-its-sole-canonical-owner
artifact: context
---

# Context

CHG-0084 claimed `src/commands/change.rs` for the change module to resolve a
finalize failure on CHG-0081. The path was already owned, by `cmd_change`.

The failure was never missing ownership. Ownership resolves against a change
declared specs, and CHG-0081 declared `change` rather than `cmd_change`, so a
path owned by another module read as unowned for that change. The correct remedy
was `correct-owner --spec cmd_change`, which is what resolved the equivalent
`src/commands/init.rs` case once a never-closed change could correct an owner.

CHG-0084 therefore fixed a problem that did not exist and introduced a real one.
The investigation that led to it read a truncated search result and concluded
that no spec claimed the path.

Lesson: a negative claim, that nothing owns a path, cannot be drawn from a
truncated search. It needs a query that would have shown the claimant.
