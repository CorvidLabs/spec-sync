## ADDED

### REQUIREMENT REQ-cmd-diff-002

`diff` SHALL NOT compute a delta from a conflicted file.

Acceptance Criteria
- A `files:` entry whose extraction unioned both sides of a conflict is reported rather than differenced, because every delta computed from that union is fiction.
- The affected paths are named, so the reader knows which entries were excluded and why.
