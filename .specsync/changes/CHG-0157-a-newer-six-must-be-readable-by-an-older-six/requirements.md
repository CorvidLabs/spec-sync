---
change: CHG-0157-a-newer-six-must-be-readable-by-an-older-six
artifact: requirements
---

# Requirements

## REQ-change-079 (new)

Evidence persisted to disk SHALL be readable by a reader that does not recognise every field it
carries, and a change recording an unrecognised workflow version SHALL be reported as written by
a newer SpecSync rather than as an invalid change state.

See `deltas/change.md` for the canonical delta.

## Deliberately unchanged

Every digest. `deny_unknown_fields` governs deserialization only; serialization is untouched,
so no preimage moves. The CHG-0068 golden vector is the check that says so.

Regenerable caches keep rejecting unrecognised shapes. Nothing about what a valid record must
contain changes — this governs what a reader must tolerate, not what a writer may emit.
