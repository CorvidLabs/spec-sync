## ADDED

### REQUIREMENT REQ-change-053

Canonical ownership of declared paths SHALL be resolved when the definition is
approved, and a change that has never closed SHALL be able to correct an
acceptance input owner without an audited reopen event.

Acceptance Criteria

- Approving a definition rejects declared paths that no declared module
  canonically owns, naming every offending path in one error.
- Paths that do not yet exist are not rejected at approve, since the owning spec
  may claim them in the same change; they remain enforced at finalize.
- A change declaring no specs is not rejected at approve, because ownership
  resolves against the change's declared specs and an empty set yields no answer.
- A change at verifying that has never closed may correct an acceptance input
  owner; its definition approval is validated in place of a reopen event.
- A change that did close continues to require an audited reopen, unchanged.
