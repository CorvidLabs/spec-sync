---
change: CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes
artifact: design
---

# Design

Two changes, together closing the state and providing its exit.

`validate_declared_path_ownership` runs in `approve_definition_with_projection`
after `validate_delta_files`. It reports every offending path at once; finding
them one per verification pass is what made the original failure expensive.

It is scoped to what it can actually resolve. Paths that do not exist yet are
skipped, since a change routinely declares a file it is about to create and the
owning spec may claim it in this change own delta. Changes declaring no specs
are skipped, because ownership resolves by searching the change declared specs
and an empty set yields no answer rather than a negative one. Both remain
enforced at finalize against full delivery evidence.

`latest_reopen_for_owner_correction` returns `Option`. A never-closed change has
no reopen because nothing reopened it; its definition approval is checked in the
reopen place, so changes that did close are unaffected.

Rejected: allowing `reopen` from Verifying. That would let a change re-enter
delivery without closing evidence, weakening a guarantee for every change to
serve one that never needed reopening.
