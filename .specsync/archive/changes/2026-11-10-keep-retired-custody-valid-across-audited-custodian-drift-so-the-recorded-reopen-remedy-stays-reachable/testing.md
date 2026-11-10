---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: testing
---

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
