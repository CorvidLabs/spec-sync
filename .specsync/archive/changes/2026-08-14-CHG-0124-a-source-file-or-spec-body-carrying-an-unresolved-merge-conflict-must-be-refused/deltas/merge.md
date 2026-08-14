## ADDED

### REQUIREMENT REQ-merge-002

Conflict detection SHALL be available as a pure content predicate and as a git-status query.

Acceptance Criteria
- The content predicate takes text and needs no repository.
- The git query distinguishes "unknown" from "no unmerged paths": a failed or absent git reports unknown, never clean.
- Conflict-marker syntax inside a fenced code block is not a conflict; blanking preserves line numbers so diagnostics still point at the real body.
