---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: research
---

# Research

What was read before choosing a shape, and what each reading settled.

**Where the binding used to live.** `definition_artifact_snapshot` enumerates `deltas/` and pushes
every file's payload into the workflow-v1 definition digest. So v1 approvals do bind delta bodies,
including the portable 5.0.1 pair path (`portable_definition_digest_pair_v501` refuses any record
that is not `workflow_version == 1`). The hole is v2-only, which is why the portable path is left
recording nothing rather than being extended.

**Which digests must not move.** `specs/change/context.md` already documents the rule:
`ChangeRecord`, `SddPolicy`, `ApprovalLedger`, `VerificationRecord` and `ChangeSequenceLedger` are
tolerant and have absorbed new fields without incident, while `ApprovedScopeV1`, `CorrectionRecord`
and `ScopedReviewRecord` are digest preimages where a field addition still changes bytes.
`ApprovalRecord` is in the first group: `approvals.json` is compared byte-for-byte against committed
copies, never re-hashed from a recomputed projection, so a field that serializes to nothing when
absent is inert for every existing ledger.

**The prior art for the compatibility shape.** `canonical_applied` is the documented precedent:
`#[serde(default, skip_serializing_if = "is_false")]`, false values omitted from new JSON, and
validation that recognizes both the omitted and the transitional encodings.
`approved_delta_digests` follows it with `Option` + `skip_serializing_if = "Option::is_none"`, which
is strictly simpler — `None` never serializes, so a record written by this binary and never approved
again is byte-identical to one written by the previous binary.

**Where deltas actually reach a canonical spec.** Two call sites, both routed through
`prepare_delta_application`: `materialize_change_deltas` (what `change check` runs) and
`accept_change_with_gate` (when `check` never materialized). Both are now gated.

**The short-circuit.** `materialize_change_deltas` returns early on `record.canonical_applied`. A
check placed after it would see a swap on the first run only and never again, so the check sits
above it: the delta must match its approval for as long as it is evidence, not just until it is
applied once.
