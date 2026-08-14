## ADDED

### REQUIREMENT REQ-git-utils-003

The absence of git history SHALL be representable as a value.

Acceptance Criteria
- "No repository" and "no commits" are distinguishable, not collapsed into one condition.
- The machine and terminal strings match those established by #558, so a reader refactored onto this helper does not change its output.
