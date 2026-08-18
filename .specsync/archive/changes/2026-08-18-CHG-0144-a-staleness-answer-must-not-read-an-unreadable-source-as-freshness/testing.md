---
change: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-stale-004 | Drill 067 gates 1, 2, 3 and 5: `stale` refuses, names the file, survives `--threshold 99`, and the markdown renderer withholds the all-clear |
| REQ-cmd-report-005 | Drill 067 gate 4, plus the measured JSON below: `stale: null`, `staleness_inconclusive: true`, `unmeasured_stale_modules: 1` |
| REQ-cmd-check-014 | `check` discloses unmeasurable cited files; its exit code on the fixture is unchanged at 1, so the disclosure is additive |
| REQ-scoring-006 | Score measured identical on both binaries at 75/100 with one penalty, not two; the git half reports withheld |
| REQ-cmd-lifecycle-004 | The no-stale guard fails on a deleted cited file rather than tolerating it as one commit under a threshold of five |
| REQ-git-utils-004 | All five staleness consumers call the one predicate; enumerated by grepping every call site of the drift primitive |

## The sandbox is the judge

Drill 067, written for this change and merged as sandbox `064bb903`:

```
candidate 06c1bcca (unfixed)   pass=5  fail=0  pending=5   FAIL
with this change               pass=10 fail=0  pending=0   PASS
```

Five gates flip. Four controls — a healthy spec still reports the all-clear,
sub-threshold drift is still fresh, real drift still reports the same number, a
`warn` project still exits 0 — pass on BOTH binaries, so the gates cannot be
satisfied by reporting everything stale or by never passing.

## Measured, old binary versus new

```
case                     OLD  FIXED  verdict
vc-healthy               0    0      IDENTICAL (control holds)
vc-subthreshold          0    0      IDENTICAL (control holds)
vc-drifted               1    1      IDENTICAL (control holds)
stale-repro              0    1      FIXED (was false green)
mixed (1 of 2 deleted)   0    1      FIXED (was false green)

deleted-source --enforcement warn   OLD=0 FIXED=0   warn projects unaffected
score                               OLD=75/100 FIXED=75/100   no double charge
report                              OLD=0 FIXED=1
```

`report --format json` on the fixture, before and after:

```
before   "stale": false   "commits_behind": 0   "staleness_inconclusive": false   "unmeasured_stale_modules": 0
after    "stale": null    "commits_behind": null "staleness_inconclusive": true    "unmeasured_stale_modules": 1
```

## Whole board

```
pass=48  fail=7  skip=0  total=55
```

Unchanged from the merged baseline. Five commands and roughly six hundred lines
changed and no drill moved, which is the check that catches collateral a
per-command test cannot.

## Suite

`cargo test` rc=0 — 2299 unit, 405 integration, 0 failures.
`cargo clippy -- -D warnings` rc=0. `cargo fmt --check` rc=0.

## Review

Two adversarial passes by an independent reviewer. The first rejected the design
outright — it established that `exists()` was the wrong oracle, that a deletion
is measurable, that the exit code had been left at 0, and found the third sibling
at `check.rs:758`. The second pass found the markdown renderer, the two remaining
siblings in `scoring` and `lifecycle`, and a dead helper documenting a floor that
was never wired. Both rounds are reflected above.
