## MODIFIED

### REQUIREMENT REQ-change-017
The lifecycle SHALL provide an audited recovery transition when accepted verification becomes stale because governed delivery inputs changed.

Acceptance Criteria
- Reopen requires an explicit non-empty human actor and reason and rejects non-stale accepted evidence.
- Reopen moves accepted evidence to verifying so strict checks remain red until a fresh verification run succeeds.
- Prior definition approval, verification, and closing approval evidence remain inspectable in append-only audit history.
- Reacceptance requires a new closing approval and does not reapply canonical deltas already accepted.

### SPEC SECTION Purpose
Provides the spec-sync 5.0 verified spec-driven development lifecycle, including audited recovery and re-verification when governed delivery inputs make accepted evidence stale.
