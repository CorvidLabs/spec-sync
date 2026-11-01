---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: requirements
---

# Requirements

### REQ-change-113

The domain reports, per affected module, the requirement IDs and spec sections that the approved
semantic delta declares, together with a three-valued integrity flag and a materialization flag.
It never reports a requirement as approved from a body that no longer matches the approval. The
full text and acceptance criteria are in `deltas/change.md`. In summary:

- Reported only for approved, implementing or verifying changes whose scope approval is current.
  These are the states and the scope half that `approval_currency` already uses.
- `delta_intact`:
  - `true` when the body matches the recorded digest. CRLF-only differences do not count.
  - `false` for a module `changed_approved_deltas` names, and for every module when the binding
    is refused as a whole (a withdrawn binding, or an unreadable delta).
  - `null` when no definition approval in the ledger recorded a digest.
- `requirements`, `sections` and `materialized` appear only when `delta_intact` is true. They are
  parsed from the same bytes that were digested.
- `materialized` is `canonical_applied` and the module not pending in
  `prepare_pending_delta_application`. It is false after a re-approved correction until the next
  check (#741).
- Only `### REQUIREMENT` and `### SPEC SECTION` item headings count. Prose does not.

### REQ-cmd-change-023

Every single-record change JSON output carries the domain list verbatim under a top-level
`approved_deltas` array. The array is always present and is `[]` when empty. Text output is
unchanged, and `summary`, the roster and transition outputs gain nothing. The full text is in
`deltas/cmd_change.md`.

### Public API (change)

`deltas/change.md` restates the `change` Public API section with three new rows:
`ApprovedDeltaSummary`, `DeltaDeclarations` and `approved_delta_summaries`. No existing row
changes.
