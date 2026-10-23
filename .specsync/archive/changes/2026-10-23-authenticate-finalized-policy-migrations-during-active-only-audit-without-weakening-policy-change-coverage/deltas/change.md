## ADDED

### REQUIREMENT REQ-change-110

The system SHALL recognize an authenticated finalized policy migration from the current delivery during active-only audit when its signed policy entry matches the current policy, without treating an unrelated historical archive or unauthenticated claim as coverage.

Acceptance Criteria
- A finalized same-delivery policy migration passes after the active workspace is removed.
- No-record, wrong-scope, fabricated archive, altered approval/review/finalization evidence, and changed policy after acceptance remain failures.
- A policy archive from a different delivery does not authorize a new policy edit.
- Existing active coverage and full-history integrity checks retain their behavior.
- Only relevant delivery archive candidates are authenticated; unrelated historical damage is not a new active-audit failure.
- The Algonest #48 reproduction passes with the candidate, and original6.1.1 still fails it. Missing history objects remain explicit verification limitations.
