---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: requirements
---

# Requirements

### REQ-change-112

The system SHALL apply the effective-contract gate to the change being verified on every
`change check`, including the first one after materialization. It SHALL record that gate's
refusal of a module the change declares as a failed verification attempt, so recorded
verification never reports a pass that the lifecycle's own gates refuse for the same tree.

Acceptance Criteria
- The first `change check` of an approved change whose declared module carries an unsuppressed
  validator warning fails and names the warning, as later checks, finalization and audit already
  do, whatever `enforcement` is configured.
- The refusal is appended to `verification-attempts.json` and projected into `verification.json`
  with `passed: false`.
- `commands[]` keeps the scoped `specsync check --spec <name> …` entry and adds
  `specsync change check <id> (effective contract: <module>, …)` with `success: false`.
- A failed attempt supersedes an earlier recorded pass. Status then names
  `specsync change check <id>` rather than review, and `change check --commit` commits nothing.
- A change whose declared modules carry no unsuppressed warning records exactly the single scoped
  command it recorded before.
- Gate failures not attributable to a module the change declares are refused before any attempt
  is recorded, as before.
- The following are unchanged:
  - `specsync check` and configured `enforcement`,
  - explicit `--strict`,
  - the gate's promotion rule and suppressions,
  - path-only scope and `--no-spec-change` changes,
  - acceptance and audit,
  - accepted or archived evidence.

### REQ-change-023 (modified)

The evidence bullet adds the gate entry: when the effective-contract gate refuses a module the
change declares, the failed attempt also records
`specsync change check <id> (effective contract: <module>, …)`, the command that reproduces that
refusal. The full restated requirement is in `deltas/change.md`, together with the matching
sentence in the spec's Contract item 5.
