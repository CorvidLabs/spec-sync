---
change: report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited
artifact: research
---

# Research

## Sweep of approval-currency readers (origin/main f240d37)

| Reader | Location | Checks scope digest | Checks delta binding |
|--------|----------|---------------------|----------------------|
| `materialize_change_deltas` (`change check`) | `src/change/sdd_version.rs:2915`, `:2922` | yes | yes |
| `accept_change` path (`finalize`) | `src/change/historical_definition_digest_matches.rs:2191`, `:2222` | yes | yes |
| `summarize_change_with_effective` (JSON status, show, list) | `src/change/historical_definition_digest_matches.rs:3413` | yes | **no** |
| `handoff_summary` (text `Handoff:` line) | `src/change/historical_definition_digest_matches.rs:3192` | yes | **no** |
| `check_project_with_command_output` (`change audit`) | `src/change/historical_definition_digest_matches.rs:3947` | yes | **no** |
| `text_mode_next_action` (text `Next:`, `ship-status` lifecycle next) | `src/commands/change.rs:807` | **no** | **no** |

`ensure_definition_approval_valid` has many other callers. Some authenticate accepted and archived
evidence, for example `src/change/consider_accepted_evidence_anchor.rs:3426`, `:3602` and `:3750`.
Another gates accepted-change interview correction
(`src/change/historical_definition_digest_matches.rs:929`). 97 of the 285 archived
`approvals.json` ledgers on f240d37 carry `approved_delta_digests`. Folding the delta comparison
into `ensure_definition_approval_valid` would therefore re-judge archived history against archive
delta paths. The fix must compose the two checks at the active-state readers only.

## Field or meaning

Nothing in `site/`, `docs/` or the specs documents `summary.approval_valid`. The code shows what
it means in practice:

- Every `!approval_valid` branch of `next_action` names `change approve`.
- `classify_handoff` reads it as "the definition changed after it was approved".

So consumers already treat it as "the approval no longer covers what is on disk". The gates that
consume the approval refuse in this state. Under workflow v1 the field already went false on a
delta edit. Correcting the value restores v1 parity for v2 changes. It does not reinterpret the
field.

A second boolean such as `delta_approval_valid` would leave `approval_valid: true` for every
existing consumer, including Chough, until each one learned to combine the two. That is the
original defect, left in place. The code's own lesson ("a predicate that returns `bool` decides
how honest every caller of it can be") argues for carrying the reason. An additive list of edited
modules carries it without a second verdict.

## Observed controls

- `plan.md` edited after approval: status valid, `check_change` passes.
- An untouched approval: status valid, `check_change` passes.
- `strip_recorded_delta_digests` already builds the pre-binding ledger shape.
- `approved_delta_body_as_a_crlf_checkout_writes_it` already builds the CRLF-only shape.
- `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding` already builds the
  withdrawn-binding shape.

All of these live in
`src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs` (3,997 lines),
so new cases can sit beside them under REQ-change-109's 5,000-line limit.

## Pre-approval dry run (no approval recorded)

This draft was copied into a disposable export of f240d37 that carries its Git history. A scratch
test then called the approval-time validators directly:

- `validate_definition`, `validate_delta_files`, `validate_declared_path_ownership` and
  `validate_artifacts` all returned `Ok`.
- Requirement evidence is present for REQ-change-111 and REQ-cmd-change-005.
- `ensure_no_delta_conflicts` passed with the strict-verification draft active beside this one.
- `prepare_pending_delta_application` wrote only the intended edits plus each spec's version
  bump and Change Log row.

`specsync check --strict --spec change --spec cmd_change` on that materialized tree reports one
finding: the documented `approval_current_for_guidance` export does not exist yet. The
implementation adds it. Nothing was approved, and no ledger was written.
