---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: docs
---

# Docs

No user-facing documentation change is required, and that is a deliberate call rather than an
omission.

The user-visible surface of this change is a verb that previously refused now succeeding, and the
refusal message becoming more precise:

    before: accepted change delivery inputs are current (exact or successor-covered);
            reopen is allowed only when delivery evidence is stale
    after:  accepted change delivery inputs are current (exact or successor-covered) and its
            verification commit is still anchored in current history; reopen is allowed only
            when accepted evidence is stale

Nobody needs new instructions to benefit: `specsync change status` already prints the reopen
command as the next action for a stranded change, and that command now works. The old behaviour
was that the tool named a verb which then refused.

`docs/ADOPTING.md` needs no edit. Its "things that will bite you" list does not mention this
deadlock, because the fix removes it.

The contract change is recorded where this repo enforces it: the `ReopenCauseV1` row in the spec's
Public API table, and the amended invariants and requirements.
