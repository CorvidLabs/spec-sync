---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-070 | `a_stale_sequence_ledger_is_raised_to_the_committed_mark_before_staging` asserts both the raise and the returned pair, so the disclosure cannot be dropped without failing. `a_sequence_ledger_ahead_of_the_committed_mark_is_left_alone` is the control and asserts the author's newer claim survives untouched; without it, "always restore the committed ledger" passes the first test and reissues taken IDs. `a_sequence_ledger_equal_to_the_committed_mark_is_not_reported` pins that equality is not a divergence. The all-staging-sites criterion is met structurally by placing the floor in `git_commit_all`, which is the sole `git add -A` in the lifecycle |
| REQ-change-071 | Gate 051's claim-F assertion goes PENDING to PASS: `audit --strict refuses a ledger below origin/HEAD with no higher workspaces on disk`. Before this half, the same gate reported `#533 claim F: audit --strict passed over a regressed ledger (local=2 origin=4)` while the commit-path assertion already passed — which is how the two halves were shown to be independent. The no-higher-workspaces-on-disk criterion is what the gate's fixture constructs deliberately, since the on-disk maximum is what the old check used. The accepted-when-at-or-above criterion is covered by the full suite: 2289 unit and 400 integration tests pass with the refusal reachable from all five `validate_change_sequences` call sites, which is the evidence it did not turn a fresh clone or unfetched branch into a refusing repository. The reuse criterion is structural — `remote_sequence_high_water` is called, not reimplemented |
| REQ-cmd-change-012 | CLI discrimination below: the same fixture, driven through the real `change check --commit`, regresses on an `origin/main` binary and is preserved with a stderr note on the fixed one, with `rc=0` in BOTH cases — which is the "does not block the author" criterion. Drill 037 covers the downstream surfaces |

## CLI discrimination — the real lifecycle, two binaries, identical fixtures

    UNFIXED   HEAD=3  worktree=1  ->  change check --commit  ->  rc=0  ->  HEAD ledger = 1
    FIXED     HEAD=3  worktree=1  ->  change check --commit  ->  rc=0  ->  HEAD ledger = 3
              stderr: note: raised the change sequence ledger from 1 to the committed 3 before
                      staging; a ledger written before the branch caught up would have
                      committed a lower high-water mark

Both exit 0. The author is not blocked in either case, which is the point — only the fixed one
preserves the mark and says what it did.

## Unit tests

    a_stale_sequence_ledger_is_raised_to_the_committed_mark_before_staging   ok
    a_sequence_ledger_ahead_of_the_committed_mark_is_left_alone              ok
    a_sequence_ledger_equal_to_the_committed_mark_is_not_reported            ok
    sequence_ledger_rejects_unacknowledged_active_and_archived_collisions    ok   (pre-existing)

## Sandbox drill 037, inverted in the same change

    UNFIXED  rc=1
      FAIL: #533 REGRESSED: committed the ledger backwards to [2 CHG-0002-stale-branch-change]
            (HEAD carried [4 CHG-0004-main-fourth-change])
      FAIL: the ledger was raised without saying so
      FAIL: #533 REGRESSED: audit --strict / finalize / ship / new each refused with the
            high-water diagnostic

    FIXED    rc=0  verdict: PASS

Six failures on the unfixed binary, all naming #533 rather than the fixture — which is the
check that the fixture repairs did not mask the signal.

## Gate 051 stayed RED after the first fix — and that is the finding

The commit-path fix alone did not flip gate 051. The gate asserts TWO behaviours and the
issue's headline is only one of them:

    PASS:         check --commit did not lower the committed high-water (4 -> 4)
    PENDING GATE: #533 claim F: audit --strict passed over a regressed ledger (local=2 origin=4)

`validate_change_sequences` compared the ledger against the highest sequence ON DISK. When the
higher-numbered workspaces are simply absent — a fresh clone, an unfetched branch — that
maximum is low and a regressed ledger passes. So a ledger regressed anywhere could be audited
clean, merged, and the next allocation would remint an ordinal the default branch had used.

Two surfaces reporting health over the same broken state, each masking the other: the commit
created the regression, the audit blessed it. Fixing either alone leaves the other.

After the second half, all eight assertions pass:

    PASS: check --commit did not lower the committed high-water (4 -> 4)
    PASS: audit --strict refuses a ledger below origin/HEAD with no higher workspaces on disk
    verdict: PASS

## Suite, with both halves

    cargo test                    rc=0   2289 unit passed, 400 integration passed, 0 failed
    cargo clippy -- -D warnings   rc=0
    cargo fmt --check             rc=0

The new refusal in `validate_change_sequences` is reachable from five call sites. The full
suite passing is the evidence it did not turn a legitimate state — a fresh clone, a detached
checkout — into a refusing one.

## Whole board

Expected: exactly one gate changes state, 051 FAIL to PASS. Drill 037 must stay PASS because
it is inverted in this same change. Any other movement is unintended reach.
