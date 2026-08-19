## MODIFIED

### REQUIREMENT REQ-change-079

Evidence persisted to disk SHALL be readable by a reader that does not recognise every field it carries, and a change recording an unrecognised workflow version SHALL be reported as written by a newer SpecSync rather than as an invalid change state.

Acceptance Criteria
- Evidence carrying a field this reader does not know is parsed rather than rejected, so an evidence shape can be extended within a major version without breaking installations already deployed.
- A change whose workflow version this reader does not support names both the cause, that a newer SpecSync wrote it, and the remedy, that the reader should be upgraded, and does not describe the record as invalid.
- The spec and source hash cache continues to reject a shape it cannot understand, because it is untracked and rebuilt from scratch on any parse failure, so discarding one costs nothing and cannot lose evidence. A file that is committed and shared is evidence regardless of what it holds, and is tolerated.
- A file read through a canonical-bytes round trip gains nothing from tolerance, because the unknown field is dropped on parse and the re-serialized bytes then differ from the bytes on disk. This limit is deliberate for the files that anchor history, and is pinned by a test rather than left to be discovered.
- Every digest is unchanged, because tolerance at read time was never part of any preimage.

## ADDED

### REQUIREMENT REQ-change-080

A persisted policy SHALL load even when it omits a field this SpecSync knows, and each omitted field SHALL take a value that enforces rather than relaxes.

Acceptance Criteria
- A policy file written before a field existed still loads, so adding a field within a major version does not make every policy written before it unreadable by the SpecSync that added it.
- An absent enablement flag reads as enabled and an absent change requirement reads as required, so a truncated or partial policy cannot silently disable enforcement.
