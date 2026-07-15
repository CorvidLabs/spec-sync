---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: requirements
---

# Requirements

### REQ-change-033

The verified lifecycle SHALL support human-authorized, append-only correction of an exact
acceptance-input canonical owner for an audited reopened, already-applied change without changing
semantic scope or replaying canonical deltas.

Acceptance Criteria

- `change correct-owner` requires an exact portable path, canonical module, non-empty actor, and
  non-empty reason.
- The target is canonical-applied, verifying through an audited reopen, and unchanged from the
  reopened definition except for validated ownership-correction entries.
- The path is already covered by the original affected paths, and the named module's current
  canonical spec explicitly owns that exact source path.
- Corrections are immutable, sequenced, definition-bound records; duplicates, removals, malformed
  values, tampering, and ambiguous ownership fail before mutation.
- Original affected specs, semantic deltas, approvals, reopen evidence, and prior verification are
  preserved byte-for-byte.
- The corrected definition requires explicit reapproval, fresh verification, and closing approval.
- Acceptance adds the corrected module only to the exact manifest entry's sorted owner set and
  never reapplies canonical deltas.
- Records without ownership corrections preserve their existing serialized bytes and digests.

### Existing REQ-change-014 regression

The accepted legacy archive baseline authority SHALL include the exact protected baseline ledger
in its acceptance manifest even though ordinary dated archive content remains volatile.
