---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: testing
---

# Testing

Every test below fails on main f7180c34 first, and that is recorded before the implementation.

1. **Closes a merged v2 change.** A `verifying` change on the remote default branch closes late:
   - it moves into the dated archive as `closed_late`;
   - `late-close.json` has every field;
   - status text and JSON say it was never re-checked and `verified: false`.
2. **Does not take the evidence read.** A merged v1 `accepted` change whose delivery-input read refuses
   (an assume-unchanged input) still closes late. Reopen fails on the same fixture.
3. **Hand-archived package.** An `accepted` package under the archive closes in place, and its other files
   keep their bytes.
4. **Unblocks finalize.** A stale `accepted` blocker makes an unrelated finalize refuse. After close-late
   on the blocker, that finalize passes.
5. **Partial finalize.** An unmerged change in the partial-finalize state is refused and left byte for
   byte. After its blocker closes late, its finalize resumes and archives it.
6. **Refusals.** Each of these writes nothing:
   - an empty `--actor`;
   - an empty `--reason`;
   - no remote default;
   - a package missing from the default branch;
   - an archived change;
   - a change already closed late.
7. **Never verified.**
   - a closed-late change is not successor coverage;
   - reopen, review and finalize refuse it;
   - audit counts it closed;
   - the archive-readiness gate passes.
8. **All or nothing.**
   - two eligible changes and one ineligible: nothing is written, and the error names the ineligible
     change;
   - `--dry-run` writes nothing and prints the plan;
   - a draft closes late and says it was never approved.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-143 | tests 1 to 8 above |

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| SHIP-12, SHIP-12.c | tests 1, 2, 3, 4 |
| SHIP-12.a | tests 1, 3, 8 (draft) |
| SHIP-12.b | tests 4, 7 |
| SHIP-12.d | test 6 (actor, reason), test 1 (recorded) |
| SHIP-12.e | tests 5, 6 |
| SHIP-12.f | test 8 |
| Failing tests first | recorded failures on f7180c34 |

## Verification commands

- `fledge lanes run verify` on the pinned rust 1.89 (rustup proxy first on PATH)
- `specsync change check <id> --commit`
- `specsync change audit --strict`
- `fledge lanes run pre-push`
