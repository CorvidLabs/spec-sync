---
change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
artifact: testing
---

# Testing

Every new test fails on 1276e314 first, and that is recorded.

1. **Scan.** A fixture with one change of each finding kind gets each finding with its kind, reason, class and
   fix. A check that errors, simulated with an assume-unchanged delivery input, becomes a finding, and the rest
   are still reported (RECOVER-3, .a, .d).
2. **Read-only.** Plain `doctor` and `doctor --fix --dry-run` leave the tree byte for byte (RECOVER-3.b).
3. **Safe fixes.** `--fix` resumes a stopped v2 finalize and a v1 archive when nothing blocks them, and runs
   the `migrate 5.0` backfill. It leaves stale verification as a finding naming `change check <id> --commit`
   (RECOVER-3.c, D2).
4. **Late close.** `--fix --close-late <id> --actor --reason` moves a merged package byte for byte to
   `.specsync/archive/closed-late/<date>-<id>/` and writes `late-close.json` with every field. It reads no
   delivery input; a v1 change whose evidence read refuses still closes. A hand-archived `accepted` package
   moves out of `archive/changes/` (SHIP-12, .a, .c).
5. **Refusals.** Each writes nothing:
   - an unmerged change, including one in the partial-finalize state;
   - no remote default;
   - an empty `--actor` or `--reason`;
   - `--close-late` without `--fix`;
   - a change already closed late or archived.

   One refusal among several IDs writes nothing and names the refused change (SHIP-12.d, .e, .f).
6. **6.3 treats it as never verified.**
   - status (text and JSON) and ship-status report the late close, and a draft says never approved;
   - reopen, review, check, accept, archive and finalize refuse it;
   - `change new` refuses its slug;
   - a finalize it blocked before now passes, and #616's shape resumes (SHIP-12.a, .b).
7. **Collision.** Late-closing a member of an acknowledged collision drops and records the acknowledgement,
   leaves the sequence untouched, and says that the ledger edit needs a covering change (D5).
8. **Compatibility.** `.github/scripts/test-closed-late-compat.sh` runs the real 6.0.0 and 6.2.0 binaries
   after `doctor --fix`, with the assertions in the design (SHIP-12.g, D4).
9. **Grammar.** `--close-late` without `--fix`, `--actor` and `--reason` is refused by argument parsing
   (REQ-cli-args-025).

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-143 | tests 4, 5, 6, 7, 8 |
| REQ-cmd-change-035 | tests 1, 2, 3, 5 |
| REQ-cli-args-025 | test 9 |

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| RECOVER-3, .a, .d | test 1 |
| RECOVER-3.b | test 2 |
| RECOVER-3.c, D2, D3 | tests 3, 5 |
| SHIP-12, .c, .e | tests 4, 5 |
| SHIP-12.a, .d | tests 4, 6 |
| SHIP-12.b | test 6 |
| SHIP-12.f | test 5 |
| D5 | test 7 |
| SHIP-12.g, D4 | test 8 (CI job) |
| REQ-cli-args-025 | test 9 |

## Verification commands

- `fledge lanes run verify` on the pinned rust 1.89
- `specsync change check <id> --commit`
- `specsync change audit --strict`
- `.github/scripts/test-closed-late-compat.sh`
- `fledge lanes run pre-push`
