# Lesson bundle: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep retired custody valid across audited custodian drift so the recorded reopen remedy stays reachable
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/retirement.rs, src/change_tests/recovery_retirement.rs
- **Acceptance**: Reopening a drifted retirement custodian no longer aborts on the retired records' custody validation; after audited re-acceptance with evolved digests, custody still validates while owners hold; owner or path loss still refuses

## Evidence

- Verification commit: `dbd3b1549c7c93e9d10e8a11ec80686fbaba1107`
- Base commit: `49d2326a8f80232b8fee94a5073c4c1a66fd2057`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

## Decisions

- Validation trusts the receipt's recorded classification as a historical fact pinned by
  committed history; it checks the classification is consistent with the receipt's own
  fields instead of re-deriving it from current evidence under current rules.
- Custody persistence is judged by path presence and unchanged signed owners in the
  custodian's current authenticated manifest. Byte/digest evolution under the same owners is
  the custodian's audited lifecycle, exactly as successor coverage already treats accepted
  evolution for ordinary changes.
- Custodian freshness against the live tree leaves retirement validation entirely; it is
  reported as terminal-evidence data on the custodian, where the reopen remedy is named.

## Ruled out

- Amending or re-signing receipts after custodian evolution: receipts are immutable and
  history-pinned by design.
- Accepting owner changes post-retirement: no byte-continuity proof exists across an
  evolution window, so owner changes still refuse.
- Chaining retirements (retiring a custodian): the replacement state rule already refuses.

## From the change's design.md

# Design

`validate_retirement_with_records` replaces its `retirement_custody` recompute and the
closing/manifest digest equality checks with `retirement_custody_holds`:

1. The replacement must exist, differ from the retired change, be Accepted or Archived, and
   not depend on the retired change (unchanged rules).
2. The replacement's recorded accepted evidence is authenticated
   (`AcceptedInputValidationMemo::authenticate`): genuine, anchored, internally bound. No
   live-tree freshness recursion.
3. For every receipt custody entry: the frozen prior manifest (itself pinned by
   `prior_manifest_digest`) carries the path at the recorded predecessor digest and owners;
   the recorded classification matches the receipt's own fields (`same-signed-owners` means
   equal owner vectors; `unchanged-exact-file-current-owner` means the recorded predecessor
   owners are exactly `@exact:delivery`, the recorded digests are equal, and the recorded
   replacement owners are non-empty named modules); the custodian's current authenticated
   manifest carries the path under exactly the recorded replacement owners.

`retire_change_with_source` keeps calling `retirement_custody` unchanged: a fresh custodian,
custody recomputed from current evidence, classifications derived under current rules.

## From the change's testing.md

# Testing

- New regression test in `src/change_tests/recovery_retirement.rs` reproducing the site's
  shape: retire under an accepted custodian, drift a custody input, assert
  `validate_retirement` holds and `reopen` of the custodian succeeds (the old code aborts
  with the custody error and `archive restored`), complete check/review/finalize, assert
  custody still holds with evolved digests. Negative: a receipt whose recorded replacement
  owners no longer match the custodian's manifest refuses validation.
- Existing retirement recovery tests (tamper, history pinning, shallow clone, owner
  classification at retire time) must keep passing unchanged.
- Full suite via `change check --commit` (native verification lane).

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-recovery-retirement-001` | `recovery_retirement_custody_survives_audited_custodian_drift_and_reacceptance` in `src/change_tests/recovery_retirement.rs` (drift keeps custody valid, reopen stays reachable, audited re-acceptance keeps custody, owner tamper refuses); the unchanged retire-time refusal tests `recovery_retirement_rejects_stale_unreviewed_circular_and_unknown_inputs_without_writes` and `recovery_retirement_requires_each_real_signed_input_and_compatible_owner`; the tamper and history-pinning tests `recovery_retirement_committed_disposition_cannot_be_resigned_by_editing_current_json` and `recovery_retirement_history_refuses_a_shallow_clone_boundary` |

## Where these lessons go

- `specs/change/change.3md`
