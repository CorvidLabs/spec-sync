---
change: CHG-0090-harden-approve-ownership-skips-and-correct-owner-provenance-comments
artifact: context
---

# Context

Adversarial review of the approve-time ownership gate and never-closed
`correct-owner` path found: (1) empty `affected_specs` silently skipped ownership
without restating that only `no_spec_change` may reach that branch; (2) comments
claimed definition-approval was "equivalent" to audited reopen provenance.

This change hardens the empty-specs branch and corrects the comments. Behavior
for justified no-spec changes is unchanged; never-closed correct-owner still uses
definition approval as the reachable substitute.
