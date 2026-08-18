---
change: CHG-0143-the-sequence-ledger-gate-must-judge-a-branch-by-its-own-history-not-by-origin
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-072 | Four unit tests plus two sandbox drill assertions, each shown below to fail against a genuinely different binary |

## Discrimination

Every assertion was run against the binary carrying the regression
(`06c1bcca`, built from a separate checkout — never by reverting files) and
against the fix.

| test | regressed binary | fixed |
|---|---|---|
| `a_branch_merely_behind_the_default_branch_is_not_refused` | **FAILED** | ok |
| `a_branch_that_lowered_the_ledger_after_diverging_is_still_refused` (control) | ok | ok |
| `a_branch_that_raised_then_rewrote_the_ledger_is_refused` | n/a — pins the new oracle | ok |
| `git_commit_all_raises_a_stale_ledger_before_staging_it` (wiring) | **FAILED** with the floor call deleted | ok |

The control passes on both binaries, so the change cannot be satisfied by
deleting the gate. The wiring test exists because `floor_sequence_ledger_to_committed`
had unit tests but nothing asserted `git_commit_all` CALLS it — deleting the call
left the entire suite green.

## Sandbox

Drill 051 gained the coverage whose absence let this regression reach main:

```
06c1bcca (regression)   pass=8  fail=1   FAIL
with the fix            pass=10 fail=0   PASS
```

Recorded as FAIL rather than PENDING GATE: a live regression on the default
branch, not a known-unfixed gap. Merged as sandbox `064bb903`.

## Whole board

```
pass=48  fail=7  skip=0  total=55
```

Unchanged from the merged baseline; the seven reds are the known PENDING GATEs
(049 050 052 053 054 056 057). No drill changed state.

## Suite

`cargo test` rc=0 — 2299 unit, 405 integration, 0 failures.
`cargo clippy -- -D warnings` rc=0. `cargo fmt --check` rc=0.
