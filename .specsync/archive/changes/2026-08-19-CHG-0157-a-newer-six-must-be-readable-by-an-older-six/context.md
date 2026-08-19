# Context

The 6.0 release candidate was audited against one question: can the schema and public API live on
6.x for years, or will something force a 7.0? Every open issue came back **additive**. Two
things did not.

## 1. `deny_unknown_fields` on persisted evidence

Seventeen structs in `src/change.rs` carried it, all persisted to disk as evidence —
`ScopedReviewRecord`, `FinalizationRecord`, `CorrectionRecord`, `ApprovedScopeV1`,
`WorkflowV2Baseline`, `LegacyArchiveBaselineV1` and the scope-adoption family among them.

An older 6.x binary therefore **cannot read a file a newer 6.x binary wrote with an added
field**. No evidence shape could be extended during 6's lifetime without breaking installations
already in the field. That is the mechanism by which "just add a field in 6.4" becomes "we need
7.0".

The tolerant structs are where growth has actually happened: `ChangeRecord`, `SddPolicy`,
`ApprovalLedger`, `VerificationRecord` and `ChangeSequenceLedger` have absorbed
`workflow_version`, `canonical_applied`, `correction_count`, `supersedes`,
`acceptance_owner_corrections` and `reopenings` without incident. Fifteen of this repository's
157 `approvals.json` files predate `reopenings` and still parse.

## 2. `workflow_version` hard-gated to `{1, 2}`

A record written with version 3 was reported as:

```
invalid change state <path>: unsupported workflow version 3
```

Indistinguishable from corruption. The operator's correct action — upgrade — was the one thing
the message did not say. A future v3 would make every older 6.x install describe the repository
as broken.

## Why in 6.0.0 and not 6.0.1

Both are inert today: nothing currently emits an unknown field, and nothing emits version 3. The
value is entirely in being present in the **oldest** 6.x binary anyone runs, which is why this
cannot be retrofitted later.
