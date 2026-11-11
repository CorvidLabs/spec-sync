---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: delta
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-079

Evidence persisted to disk SHALL be readable by a reader that does not recognise every field it carries, and a change recording an unrecognised workflow version SHALL be reported as written by a newer SpecSync rather than as an invalid change state. A lifecycle record or ledger that SpecSync rewrites in place SHALL keep every member it does not recognise, so a record a newer SpecSync wrote loses nothing when an older one updates it.

Acceptance Criteria
- Evidence carrying a field this reader does not know is parsed rather than rejected, so an evidence shape can be extended within a major version without breaking installations already deployed.
- A change whose workflow version this reader does not support names both the cause, that a newer SpecSync wrote it, and the remedy, that the reader should be upgraded, and does not describe the record as invalid.
- The spec and source hash cache continues to reject a shape it cannot understand, because it is untracked and rebuilt from scratch on any parse failure, so discarding one costs nothing and cannot lose evidence. A file that is committed and shared is evidence regardless of what it holds, and is tolerated.
- A file read through a canonical-bytes round trip gains nothing from tolerance, because the unknown field is dropped on parse and the re-serialized bytes then differ from the bytes on disk. This limit is deliberate for the files that anchor history, and is pinned by a test rather than left to be discovered.
- Every digest is unchanged, because tolerance at read time was never part of any preimage.
- A rewrite in place of `state.json`, `approvals.json`, `verification-attempts.json`, `review-attempts.json`, `corrections.json`, a relay repair's `review.json`, the archived `state.json` or the change-sequence ledger keeps each member this SpecSync does not know, with its exact text: members of the root, and members of an array entry whose known content the rewrite left unchanged. An entry the rewrite changed or replaced, and a record written afresh, take none, so no record gains another record's members.
- The kept members change nothing this SpecSync reads and no digest; a rewrite that cannot keep them is refused, naming the file and the members, instead of dropping them.
