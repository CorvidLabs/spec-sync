---
id: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
state: archived
type: feature
base_commit: a80748ff44ff42cd7578ac0f7cb8ba7e080ab183
---

# Report the requirement IDs an approved semantic delta declares, per module, in change status and show JSON

## Intent

Report the requirement IDs an approved semantic delta declares, per module, in change status and show JSON

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- After a workflow-v2 change is approved, `specsync change status <id> --json` and `specsync change show <id> --json` carry a top-level `approved_deltas` array with one entry per affected module; for the issue #826 reproduction the `shipping` entry reports `delta_intact: true`, `requirements.added: ["REQ-shipping-002"]`, `sections.modified: ["Public API"]` and `materialized: false`.
- Editing an approved delta body after approval makes that module's entry report `delta_intact: false` with no `requirements`, `sections` or `materialized`, so no requirement is reported as approved from an unapproved body; the module is the one `summary.changed_approved_deltas` names, `summary.approval_valid` is false and `summary.next_action` names re-approval, as on origin/main a80748f.
- Converting an approved delta to CRLF line endings without other edits keeps `delta_intact: true`; restoring or re-approving an edited delta returns it to true; a later definition approval that withdraws a recorded delta binding reports every module `delta_intact: false`; a ledger that never recorded delta digests reports `delta_intact: null` and no requirements, never false.
- After a passing `change check` the entry reports `materialized: true`; after a corrected delta is re-approved following materialization it reports `materialized: false` until the next `change check` applies it.
- Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` item headings are reported; a requirement ID mentioned in prose inside a delta body is not.
- `approved_deltas` is `[]` for draft, accepted and archived changes and for an approved change whose scope approval is stale; it is always present in single-record change JSON; no existing JSON key, `summary` field, `change list --json` output, text output, persisted ledger, digest or materialization behavior changes.
- Regression tests fail on origin/main a80748f and pass with the change: unit discriminators and labelled controls for each case above, and an integration test that drives `change status --json` and `change show --json` through the built binary.

## No-spec Rationale

Not applicable
