---
change: CHG-0148-a-reopened-change-must-be-closeable-again
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-073 | Gate drill 049 and pin drill 013, each shown below to fail against the unfixed binary and pass against this one |

## Discrimination

Both drills were run against a binary built from a separate checkout at
`a9ebf7cf` — never by reverting files in place.

| drill | unfixed `a9ebf7cf` | with this change |
|---|---|---|
| 049 (gate) | `pass=11 fail=0 pending=2` — FAIL | `pass=12 fail=0 pending=0` — PASS |
| 013 (pin, inverted) | `FAIL (#540 regression): a reopened change could not be finalized again` | `PINNED (#540 fixed): archived once, workspace cleared` |

Drill 049's two gates were exactly the dead end: *"finalize after reopen refused
and left state=accepted archives=0"* and *"left an accepted dead-end:
review/reopen/check all refuse"*. Eleven surrounding assertions — the entire
first close and the post-reopen sequence — pass in both states and act as the
controls.

## The pin drill needed its assertion changed, not just inverted

Drill 013 is below 044 and does not self-flip. Its original block read
`.specsync/changes/<id>/state.json` to report the stranded state. That path only
exists while the bug does: a repaired finalize archives the change and clears
the workspace. The first inversion crashed with `FileNotFoundError` against the
fixed binary.

It now asserts the repaired shape instead — exactly one archive package for the
change, and no surviving active workspace — which cannot accidentally pass on a
stranded change.

## Whole board

```
pass=50  fail=6  skip=0  total=56
```

Two drills changed state, both expected: 049 (the gate this closes) and 013 (its
pin, inverted in the same work). The six remaining reds are unchanged:
050 052 053 054 056 057.

## Note on the binary under test

Partway through this work an unrelated `git stash -u` removed the fix from the
working tree while the built binary retained it. The drills continued to pass,
truthfully but meaninglessly. The source was restored, rebuilt, and both drills
re-run against the new binary before any of the results above were recorded.
