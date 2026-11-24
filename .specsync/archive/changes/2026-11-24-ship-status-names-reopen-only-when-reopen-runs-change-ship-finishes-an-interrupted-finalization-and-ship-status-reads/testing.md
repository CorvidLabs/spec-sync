---
change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
artifact: testing
---

# Testing

- **Unit, failing first** (`src/commands/change.rs`): the two kept #922 tests (an archived change not told to reopen for unrelated drift; an unreadable `verification.json` not told to reopen), plus stale accepted and archived changes told to reopen in the product-tip stage and `ship_next`, never `change check`; #933 on a real interrupted finalization (a test-only stop after acceptance), ready and archived by `change ship`, an open task named with `specsync change ship <id>`, and an accepted change finalize would not resume not reported ready; #938 counting freeze evaluations per report, with and without a freeze.
- **Through the binary**: at least one test per fix, `change ship-status` and `change ship` as a person runs them.
- **Agreement**: in each #922 case, `change reopen` refuses or accepts as ship-status says.
- **#1038**: the assertion in `src/change_tests/historical_reconciliation.rs`.
- **Gates**: `change audit --strict` with origin/main fetched, and `fledge lanes run verify`.
