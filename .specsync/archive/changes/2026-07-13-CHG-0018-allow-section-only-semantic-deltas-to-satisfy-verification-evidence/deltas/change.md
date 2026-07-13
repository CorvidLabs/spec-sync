## ADDED

### REQUIREMENT REQ-change-019

Verification SHALL recognize a non-removed requirement or spec-section delta item as semantic acceptance evidence when observable acceptance criteria are present.

Acceptance Criteria

- A section-only modified delta can pass with an empty requirement-ID list.
- Requirement evidence mapping remains mandatory for every collected requirement ID.
- A failed configured command, missing semantic acceptance evidence, and missing requirement evidence produce distinct diagnostics.
