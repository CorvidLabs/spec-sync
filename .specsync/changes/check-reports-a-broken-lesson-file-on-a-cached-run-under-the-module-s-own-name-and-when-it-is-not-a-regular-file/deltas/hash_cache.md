## ADDED

### REQUIREMENT REQ-hash-cache-011

A spec's cached verdict SHALL be bound to the lesson files beside it: every `.3md` entry in the spec's directory is an input of the validation snapshot and of the changed-spec classification, so adding, editing, removing or replacing one re-validates the spec instead of skipping it or replaying the earlier verdict.

Acceptance Criteria
- After a clean `check`, a `check` with no flags re-validates a spec whose `<module>.3md` was added or edited, and reports it when it does not parse.
- A snapshot recorded before a `.3md` beside the spec was added, edited, removed or replaced by a directory is not replayable; with no such change it is.
