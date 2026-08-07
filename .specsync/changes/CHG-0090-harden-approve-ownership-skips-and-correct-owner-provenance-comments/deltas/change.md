## MODIFIED

### REQUIREMENT REQ-change-053

Canonical ownership of declared paths SHALL be resolved when the definition is
approved, and a change that has never closed SHALL be able to correct an
acceptance input owner without an audited reopen event.

Acceptance Criteria

- Approving a definition rejects declared paths that no declared module
  canonically owns, naming every offending path in one error.
- Paths that do not yet exist are not rejected at approve, since the owning spec
  may claim them in the same change; they remain enforced at finalize.
- A change with justified `no_spec_change` (empty declared specs) is not
  ownership-rejected at approve, because there is no owner set to resolve
  against; finalize still enforces production ownership. Empty specs without
  that justification fail closed at definition validation and at ownership
  validation.
- A change at verifying that has never closed may correct an acceptance input
  owner under a currently valid definition approval. That substitute is for
  guided-path reachability, not audit-equivalent provenance to an
  Accepted→reopen cycle.
- A change that did close continues to require an audited reopen, unchanged.
