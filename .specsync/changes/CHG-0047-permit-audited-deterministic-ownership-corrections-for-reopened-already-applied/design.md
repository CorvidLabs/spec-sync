---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: design
---

# Design

Add an omitted-when-empty `acceptance_owner_corrections` field to `ChangeRecord`. Each
`AcceptanceOwnerCorrection` contains schema version, monotonic sequence, normalized exact path,
canonical module, actor, reason, and timestamp. The ordered records are part of the normal
definition digest; legacy records with an absent field deserialize to an empty vector and retain
byte-identical definition evidence.

Expose `change correct-owner <id> --path <path> --spec <module> --actor <actor> --reason <reason>`.
The transition accepts only a canonical-applied record in `verifying` with a valid latest reopen
event. Before writing, reconstruct the reopened definition by clearing ownership corrections and
require it to match the prior verification contract. Normalize and validate the exact path, require
the original affected-path scope to cover it, resolve the module through the canonical registry,
and parse the trusted current spec bytes to prove its frontmatter owns the path. Reject exact-owner
pseudo-modules, duplicates, empty audit fields, unsafe paths, and any pre-existing unrelated
definition mutation. Prepare the state and rendered Markdown atomically; do not touch approval,
verification, correction, or delta files.

Definition approval signs the corrected state through the existing digest. Verification binds the
same digest. Acceptance permits the already-applied definition difference only when removing every
validated ownership correction reproduces the latest reopen contract. Manifest construction adds
the correction module only for the correction's exact path, revalidates current canonical
ownership, sorts and deduplicates owners, and prepares no semantic-delta writes. Status, strict
checking, history reconstruction, and archive validation share the same correction validator.
