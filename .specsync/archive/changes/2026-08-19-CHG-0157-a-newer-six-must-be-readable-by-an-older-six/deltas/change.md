## ADDED

### REQUIREMENT REQ-change-079

Evidence persisted to disk SHALL be readable by a reader that does not recognise every field it carries, and a change recording an unrecognised workflow version SHALL be reported as written by a newer SpecSync rather than as an invalid change state.

Acceptance Criteria
- Evidence carrying a field this reader does not know is parsed rather than rejected, so an evidence shape can be extended within a major version without breaking installations already deployed.
- A change whose workflow version this reader does not support names both the cause, that a newer SpecSync wrote it, and the remedy, that the reader should be upgraded, and does not describe the record as invalid.
- Regenerable caches continue to reject a shape they cannot understand, because discarding and rebuilding one costs nothing and cannot lose evidence.
- Every digest is unchanged, because tolerance at read time was never part of any preimage.
