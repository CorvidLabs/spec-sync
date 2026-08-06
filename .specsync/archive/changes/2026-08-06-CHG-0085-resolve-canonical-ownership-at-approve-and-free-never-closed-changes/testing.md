---
change: CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-053` | `approve_rejects_a_declared_path_owned_by_an_undeclared_module` declares a spec owning one path and not another, asserting the unowned path is reported and the owned one is not. `never_closed_verifying_change_corrects_an_owner_without_a_reopen` corrects an owner on a verifying change with no reopen event. Both fail on the unfixed code with the exact production messages. Suites: 2,185 unit and 333 integration. Sandbox drill 037 passes at its earliest branch, approve refusing the cross-module path before any verification runs. Confirmed on the real stranded change: CHG-0081 accepted `correct-owner` and finalized. |

## Manual verification

- CHG-0081, stranded through four verification passes, corrected and archived.
- CHG-0082 archived by the same sequence.

## Notes

The first implementation over-rejected twice, and the suites caught both: paths
not yet created, and every `--no-spec-change` change. The unit test also modelled
the wrong shape and passed anyway, which is why it now asserts that the owned
path is absent from the error rather than only that the unowned one is present.
