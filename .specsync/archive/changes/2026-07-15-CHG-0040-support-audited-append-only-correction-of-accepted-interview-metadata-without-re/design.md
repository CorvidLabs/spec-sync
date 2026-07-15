---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: design
---

# Design

## Data model

`CorrectionField` is a snake-case serialized enum with `public_contract` and `architecture_risk`.
`CorrectionRecord` is a versioned immutable event containing the change ID, sequence, field, original
value, prior effective value, corrected value, actor, reason, timestamp, prior/corrected portable
metadata-view digests, monotonically added artifacts, the superseded definition and closing
approvals, and the prior verification. `CorrectionLedger` is a versioned ordered event list stored as
`corrections.json`. `CorrectionResult` returns the persisted change, event, effective definition,
summary, and complete history.

The original `ChangeRecord.answers` and `selected_artifacts` remain historical facts. Internal
projection folds valid records in sequence to produce `EffectiveChangeDefinition` with effective
answers and the union of original plus correction-added artifacts.

## Digest model

1. Start a domain-separated metadata-view digest from the change ID, original supported answers,
   and original selected artifacts.
2. For each event, validate schema, sequence, field/value grammar, original and prior-effective
   values, actor/reason, and prior digest before framing its immutable correction payload.
3. Compare the resulting digest with the event's corrected-view digest.
4. Include the validated correction ledger and effective artifact files in the normal portable
   definition digest used by approval and verification.

This avoids a self-referential digest while binding both the audit chain and the complete current
definition.

## Transition

```text
accepted + canonical applied
  -> validate accepted evidence and integrated history
  -> append correction + scaffold monotonic artifacts atomically
  -> verifying (definition approval stale)
  -> complete artifacts
  -> approve definition
  -> verify implementation/effective contracts
  -> accept with no delta preparation
  -> accepted
```

Only accepted active workspaces can append a correction, so repeated correction is naturally gated
by reacceptance. Any failed validation or multi-file write leaves the prior accepted workspace
unchanged.

## CLI and projection

`change correct <id> <field> <value> --actor <human> --reason <text>` maps directly to the typed
domain transition. JSON returns typed persisted records. Text output names the old/new value, added
artifacts, and next action. Show/status include the effective value and ordered history rather than
silently presenting the original answer as current.
