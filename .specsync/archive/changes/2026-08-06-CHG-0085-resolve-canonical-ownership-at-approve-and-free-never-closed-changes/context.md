---
change: CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes
artifact: context
---

# Context

Three blockers surfaced in one session, all enforced at finalize, all knowable
far earlier. The archive scope guard and the CLI-wiring ownership gap were the
other two.

The pattern is the finding. Late enforcement is not merely slow: it converts a
two-second rejection into an unrecoverable state, because the recovery commands
are themselves scoped to changes that have already closed.

Two fixtures in the unit suite and one in the integration suite described
projects where production source belonged to no spec at all — the exact state
that produces the dead end. They passed because nothing checked. Fixtures that
do not resemble real projects cannot fail the way real projects do.
