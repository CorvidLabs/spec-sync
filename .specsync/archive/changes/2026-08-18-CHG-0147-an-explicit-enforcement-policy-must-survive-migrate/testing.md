---
change: CHG-0147-an-explicit-enforcement-policy-must-survive-migrate
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-config-012 | Sandbox drill 068, written before the fix and shown below to fail against the unfixed binary; 119 config unit tests unchanged |

## The sandbox is the judge

Drill 068 was written FIRST, and confirmed red on `a9ebf7cf` before any code
changed:

```
unfixed a9ebf7cf   pass=4  fail=0  pending=2   FAIL
with this change   pass=6  fail=0  pending=0   PASS
```

The behavioural line, both binaries:

| | check before migrate | check after | enforcement lines written |
|---|---|---|---|
| unfixed | rc=0 | rc=1 | 0 — key dropped |
| fixed | rc=0 | rc=0 | 1 — value stated |

## Controls, green on BOTH binaries

- an explicit `strict` survives migrate
- a project that never set enforcement is unaffected
- a tree with nothing to report still exits 0 after migrate

So the fix cannot be satisfied by never migrating, or by always writing
`strict`, or by making migrate a no-op.

## Fixture design

The tree cites a source file that does not exist, which is a validation ERROR.
That is deliberate: undocumented exports are only WARNINGS and `strict` passes
those, so a warning-based fixture cannot distinguish the two policies and would
have reported success against the unfixed binary.

## A false pass, found and fixed in the drill itself

The first version of drill 068 reported PASS for "the migrated config still
states its enforcement explicitly" against the unfixed binary, while its own log
line printed `enforcement lines in migrated config=0` directly above. Cause:
`grep -c` prints `0` AND exits 1 on no matches, so `|| echo 0` appended a second
zero and the variable stopped comparing equal to `"0"`. Counting with `wc -l`
fixed it and the assertion then failed correctly.

Recorded because it is the same defect class as the bug under repair, one level
up in the tooling.

## Whole board

```
pass=50  fail=7  skip=0  total=57
```

Exactly one drill changed state — 068, the gate this closes. The seven reds are
the known PENDING GATEs (049 050 052 053 054 056 057).

## Suite

`cargo test --bin specsync config` — 119 passed, 0 failed.
