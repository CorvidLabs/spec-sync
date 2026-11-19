## ADDED

### REQUIREMENT REQ-hash-cache-006

A validation snapshot SHALL be bound to the spec's history fragments: the snapshot input digest SHALL frame the name and content of every entry in the spec's `history/` directory, so adding, editing or removing a fragment re-validates the spec instead of replaying the earlier verdict.

Acceptance Criteria
- A snapshot recorded before a fragment was added, edited or removed is not replayable; with no fragment change it is.
