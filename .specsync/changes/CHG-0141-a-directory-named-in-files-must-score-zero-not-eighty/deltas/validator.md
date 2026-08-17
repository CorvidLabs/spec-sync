## ADDED

### REQUIREMENT REQ-validator-042

Validation SHALL decide that a `files:` entry is a directory using the shared export-scan predicate, rather than its own test.

Acceptance Criteria
- The directory refusal is reached through the same predicate every other command uses, so `check` and `score` cannot disagree about whether one path is a directory.
- A directory export scan contributes no symbols, exactly as an unreadable one does, without the two being reported as the same condition.
- The existing refusal text and exit status for a directory mapping are unchanged.
