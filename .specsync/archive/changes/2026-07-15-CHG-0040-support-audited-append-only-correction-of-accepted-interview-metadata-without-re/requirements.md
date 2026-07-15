---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: requirements
---

# Requirements

### REQ-CHG-0040-001

The lifecycle SHALL correct supported accepted interview metadata only through an append-only,
human-authorized audit event.

Acceptance Criteria

- `public_contract` and `architecture_risk` are the only initially supported fields.
- The actor, reason, original value, prior effective value, corrected value, timestamp, and portable
  prior/corrected metadata-view digests are persisted.
- The original `ChangeRecord` answers, selected artifacts, approvals, and prior verification remain
  inspectable and byte-for-byte attributable.
- Empty actors or reasons, unsupported fields, invalid values, no-op corrections, and non-accepted
  changes are rejected without mutation.

### REQ-CHG-0040-002

The lifecycle SHALL derive and validate one fail-closed effective definition from original metadata
and the ordered correction ledger.

Acceptance Criteria

- Every correction chain starts from the original answer and each event's prior value and digest
  match the preceding effective view.
- Correcting a classification to `yes` monotonically adds its deterministic artifacts; no correction
  can remove an artifact or reduce already-recorded scrutiny.
- Missing, malformed, truncated, reordered, unsupported, or digest-tampered ledgers fail strict
  checking, approval, verification, and acceptance.
- Definition digests include the validated correction ledger and effective artifact set using
  repository-relative portable paths.

### REQ-CHG-0040-003

The correction transition SHALL require fresh evidence without replaying canonical semantic deltas.

Acceptance Criteria

- A successful correction moves `accepted` to `verifying` while keeping canonical application true.
- Fresh definition approval, successful verification, and closing approval are required before the
  change can return to `accepted`.
- Acceptance prepares no canonical delta application for an already-applied corrected change.
- Repeated corrections are supported only after each prior correction has been reaccepted, and all
  prior correction and gate evidence remains inspectable across squash-integrated history.

### REQ-CHG-0040-004

The CLI SHALL expose equivalent deterministic text and JSON correction workflows.

Acceptance Criteria

- `specsync change correct <id> <field> <value> --actor <human> --reason <text>` declares all audit
  inputs explicitly.
- Correct, show, and status output expose original/effective values, the ordered correction records,
  actors, reasons, timestamps, digests, gate health, added artifacts, and the next required action.
- Parser and domain errors are specific, non-zero, and leave lifecycle files unchanged.
- Public workflow and CLI documentation distinguish metadata correction from delivery-only reopen.
