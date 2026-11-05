---
change: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text
artifact: design
---

# Design

- New serde-optional `ApprovalRecord.approved_delta_bases`. It maps a readable key such as `cli: REQUIREMENT REQ-cli-007` to a framed digest (domain `specsync.approved-delta-base.v1`) of the canonical block at approval: line endings folded, surrounding blank lines trimmed, absence recorded as absence.
- Writers:
  - `append_approval` (definition gate only) and the `--portable-5-0-1` pair read the bases beside the bodies.
  - The normalizing approval in `accept_change` carries the superseded approval's bases forward rather than re-reading an already-applied tree.
- `ensure_approved_delta_bases_current` runs in `materialize_change_deltas` directly after the body guard, above the `canonical_applied` short-circuit and before any write. Audit runs it only once the body guard holds.
- An item that is already applied is converged and never refused. A missing key, or a ledger without bases, is unknown, not violated (same reading as REQ-change-089/090).
