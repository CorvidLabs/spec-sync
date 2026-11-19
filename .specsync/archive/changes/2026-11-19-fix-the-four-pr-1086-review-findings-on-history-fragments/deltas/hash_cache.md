## MODIFIED

### REQUIREMENT REQ-hash-cache-006

A validation snapshot SHALL be bound to the spec's history fragments: the snapshot input digest SHALL frame the name, type and content of every entry in the spec's `history/` directory, and the type of the directory itself, so adding, editing, removing or replacing a fragment, including swapping it for a symlink to the same bytes, re-validates the spec instead of replaying the earlier verdict.

Acceptance Criteria
- A snapshot recorded before a fragment was added, edited or removed is not replayable; with no fragment change it is.
- A snapshot recorded before a fragment was swapped for a symlink to a copy of the same bytes is not replayable.
