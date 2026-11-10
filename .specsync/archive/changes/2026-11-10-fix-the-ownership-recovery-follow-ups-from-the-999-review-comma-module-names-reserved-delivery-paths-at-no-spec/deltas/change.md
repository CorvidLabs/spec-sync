---
change: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec
artifact: delta
module: change
---

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
- A declared path that is a directory is not file-level ownership-checked at
  approve; its descendant files remain checked at finalize.
- Recognized reserved delivery paths (for example files under `.github/` or
  `docs/`, and the protected SDD and root metadata files) are signed with the
  exact delivery owner by design and finalize accepts them, so they are not
  ownership refusals at approve and not `correct-owner` discovery candidates.
- A change with justified `no_spec_change` and no declared specs is
  ownership-checked at approve as well: an existing production-source path in
  its scope that is not recognized reserved delivery metadata has no declared
  owner, so it is rejected, naming the module that owns it, before anyone
  approves or reviews it (#986, #956). Empty specs without that justification
  still fail closed at definition validation and at ownership validation.
- A change at verifying that has never closed may correct an acceptance input
  owner under a currently valid definition approval. That substitute is for
  guided-path reachability, not audit-equivalent provenance to an
  Accepted→reopen cycle.
- A change that did close continues to require an audited reopen, unchanged.
- Owner recovery never prints a command that cannot run as printed: when a
  module name cannot round-trip through the comma-separated `affected_specs`
  answer, the guidance names the modules in prose instead. When the spec
  listing exceeds its bound, the recovery says the listing is incomplete rather
  than omitting an owner. A recovery that replaces `affected_specs` says the
  constraints answer must be renewed before approval. Owner discovery includes
  registry-registered canonical specs outside the specs directory.
