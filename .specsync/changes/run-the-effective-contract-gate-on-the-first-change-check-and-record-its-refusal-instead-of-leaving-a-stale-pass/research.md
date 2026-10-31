---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: research
---

# Research

## Which gate decides what

| Gate | Where | Rule for an unsuppressed warning in a declared module | Sees the change on its first check? |
|------|-------|-------------------------------------------------------|-------------------------------------|
| Scoped spec↔code pass | `evaluate_spec_code_sync`, `src/change/sdd_version.rs:3268` | fails only under explicit `--strict` | yes |
| Effective-contract gate in verification | `verify_change_locked`, `src/change/sdd_version.rs:3040` | always fails | **no**: excluded while `canonical_applied` and not yet verifying |
| Effective-contract gate in acceptance (`finalize`) | `src/change/historical_definition_digest_matches.rs:2226` | always fails | yes, the change is verifying |
| Effective-contract gate in `change audit` | `src/change/historical_definition_digest_matches.rs:4036` | always fails | only once verifying |
| `specsync check` under `enforcement = "strict"` | `compute_exit_code`, `src/commands/mod.rs:1204` | passes (warning) | not applicable |
| `specsync check --strict` | same | fails | not applicable |

The lifecycle already decides that a declared module's warnings block delivery, because
finalization refuses them. The defect is the one run that does not ask: the first check, which is
the only run that records a pass. "The gate that never asks reads its silence as a pass" is a
shape this module's lessons have recorded before (#728, and the `ship-status` review-currency
fix).

## Alternatives considered

1. **Honor configured `enforcement = "strict"` by verifying with `--strict`.** This was the
   report's proposed direction. Rejected because it is the wrong mechanism:
   - Configured strict enforcement passes this tree under `specsync check`. Making `change check`
     stricter than `specsync check` for the same configuration gives `enforcement` two meanings.
   - Under `enforcement = "warn"` the defect stays: the probe shows the first check still records
     a pass the gate refuses.
2. **Stop the gate from promoting undocumented-export warnings.** Rejected because it changes
   what finalization and audit accept. Changes could then finalize with undocumented exports in
   modules they declare. That contradicts the gate's documented "every validator warning is
   promoted" rule, whose narrow suppressions REQ-change-063 defines. It is a product decision,
   not a defect fix.
3. **Have `change status` run the gate on every call.** Rejected. It adds an effective-contract
   validation to the hottest read path, and it is not needed once the first check runs the gate
   and a refusal is recorded. The remaining exposure is a pass recorded by a pre-fix binary,
   which the next `change check` supersedes.
4. **Relax the filter for every caller.** Rejected. `change audit` would also start judging
   implementing records with already-applied deltas. That is a wider behavior change than this
   defect needs.

## Naming the refusal in evidence

REQ-change-023 makes evidence "the scoped command the verdict was reached under", and the F4
status fix (`status_does_not_advertise_strict_unless_strict_was_requested`) refused to record a
`--strict` the pass never ran.

`specsync check --spec <m> --strict` is therefore not an honest name for the gate's refusal:

- The gate suppresses stub sections no active change authored, and `--strict` does not.
- The gate validates the effective contract with other active deltas replayed, and
  `specsync check` validates only the canonical file.

`specsync change audit` reproduces the gate but reports project-wide findings.

`specsync change check <id> (effective contract: <module>, …)` names the command that reproduces
the verdict exactly. It annotates the gate the same way the existing
`specsync check (no spec in scope)` annotates an empty scope.

## Evidence-shape compatibility

- `VerificationRecord.commands` is already a list. `passed` is the conjunction of entry successes
  at `sdd_version.rs:3059`.
- Status advertises `commands[0]` (`summarize_change_with_strict`), and the existing test compares
  `commands[0]` only, so it keeps holding.
- Archived and accepted records are never re-verified, so no persisted evidence changes meaning.

## Pre-approval dry run (no approval recorded)

This draft was copied into a disposable export of f240d37 that carries its Git history. A scratch
test then called the approval-time validators directly:

- `validate_definition`, `validate_delta_files`, `validate_declared_path_ownership` and
  `validate_artifacts` all returned `Ok`.
- Requirement evidence is present for REQ-change-112 and REQ-change-023.
- `ensure_no_delta_conflicts` passed with the status-approval draft active beside this one.
- `prepare_pending_delta_application` wrote only the intended requirement and Contract edits plus
  the version bump and Change Log row.

`specsync check --strict --spec change --spec cmd_change` passes on that materialized tree.
Nothing was approved, and no ledger was written.
