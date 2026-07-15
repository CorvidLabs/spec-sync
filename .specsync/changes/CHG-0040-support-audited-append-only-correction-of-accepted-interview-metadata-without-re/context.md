---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: context
---

# Context

GitHub issue #360 identifies a gap between two valid safety rules. Accepted change definitions
must not be rewritten, but review can prove that a deterministic interview answer was historically
wrong. Today `change reopen` correctly handles stale delivery evidence and rejects definition
mutation, leaving users with no supported way to correct the historical classification.

## Decisions

- Add `change correct` only for accepted, canonically applied changes. It is not a general editing
  escape hatch.
- Initially support only the closed `public_contract` and `architecture_risk` fields, with normalized
  `yes`/`no` values. Scope, acceptance criteria, affected modules, and arbitrary answers require a
  successor change.
- Preserve `ChangeRecord.answers`, the original selected artifacts, every approval, and prior
  verification. Store corrections in a separate versioned append-only `corrections.json` ledger.
- Resolve an effective definition by replaying the validated correction chain. A correction to
  `yes` can add the artifacts selected by the deterministic interview, but corrections never remove
  artifacts or prior evidence.
- Bind every event to portable, domain-separated prior and corrected effective-metadata digests.
  Definition approval continues to bind the complete effective artifact and semantic-delta view.
- Move `accepted` to `verifying`, preserve `canonical_applied: true`, and require fresh definition
  approval, successful verification, and closing approval. Acceptance therefore uses the existing
  non-replay path and cannot apply the canonical delta twice.
- Persist the ledger, lifecycle transition, and newly required artifact templates atomically.
  Malformed, truncated, unsupported, or digest-inconsistent correction history fails closed.

## Compatibility

Legacy active and archived workspaces without `corrections.json` have an empty correction history
and retain their current digest behavior. Existing `change reopen` remains the recovery operation
for delivery-only staleness; `change correct` is the explicit operation for supported historical
classification errors.

## Non-goals

- Rewriting an archived workspace.
- Correcting free-form acceptance criteria, affected scope, change type, or custom interview fields.
- Removing artifacts or lowering evidence already required by the original definition.
- Replaying canonical semantic deltas or claiming earlier CI verified a corrected definition.
