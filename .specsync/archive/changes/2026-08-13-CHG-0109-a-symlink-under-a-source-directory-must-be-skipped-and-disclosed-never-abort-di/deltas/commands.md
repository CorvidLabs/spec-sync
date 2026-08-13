## ADDED

### REQUIREMENT REQ-commands-007

Strict validation SHALL refuse to report success for a tree whose coverage excluded skipped symlinked entries.

Acceptance Criteria
- Strict mode exits non-zero when any entry was skipped, naming how many.
- Bare validation continues to exit zero and only reports the exclusion.
- Both the text and machine-readable exit paths apply the same rule.
