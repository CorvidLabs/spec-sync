---
change: CHG-0015-add-audited-stale-accepted-change-reopening
artifact: requirements
---

# Requirements

### REQ-change-017

The lifecycle SHALL provide an audited recovery transition when accepted verification becomes stale because governed delivery inputs changed.

#### Acceptance Criteria

- An explicit non-empty human actor and reason are mandatory.
- Current accepted evidence is rejected.
- Reopen remains fail-closed until fresh verification.
- Prior verification and closing approval remain inspectable.
- Reacceptance requires a new closing approval and never reapplies canonical deltas.
- Text and deterministic JSON clients expose equivalent behavior.
