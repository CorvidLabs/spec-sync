---
change: CHG-0020-harden-reopened-acceptance-compatibility-and-canonical-governance
artifact: design
---

# Design

Reuse `definition_digest_matches` for the pre-reopen contract comparison so omitted and transitional explicit-false encodings remain equivalent. Require canonical successors to represent semantic spec changes by excluding `no_spec_change` records. Include canonical-applied records only while they are verifying in effective-contract module selection, and validate their current canonical modules without replaying already-applied deltas.
