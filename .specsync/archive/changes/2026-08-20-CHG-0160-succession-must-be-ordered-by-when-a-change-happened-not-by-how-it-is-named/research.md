# Research

## Six sites derived succession order from the ID string

`succession_change_key(id) -> (u64, String)` was `(change_sequence(id).unwrap_or(u64::MAX), id)`.
Its callers:

| Site | Used for |
|---|---|
| `:2447` | sort `record.supersedes` |
| `:7460` | enforce a strict sort over the same list |
| `:7519` | the happens-before guard: predecessor must sort before successor |
| `:10888` | sort semantic-succession tuples into a digest preimage |
| `:14722` | successor coverage: a later accepted change covers a drifted delivery input |
| `:14842` | the same, for the sequence-ledger owner |

Under an identity scheme without ordinals, `change_sequence` returns `None` for every ID, every
key collapses to `(u64::MAX, id)`, and all six degrade to plain lexicographic order — silently.
No error, no compile failure, no failing test. "Supersedes" would come to mean "sorts
alphabetically after".

## It is already wrong, at five digits

Two sorts run over the same list and disagree:

- `:2447` / `:7460` — numeric then lexicographic
- `:9308` `approved_scope` — `predecessor_id.cmp()`, pure lexicographic, and the result is
  serialized into `ApprovedScopeV1` and hashed into `scope_digest`

```
numeric:        CHG-9999  <  CHG-10000
lexicographic:  CHG-10000 <  CHG-9999
```

They agree only while every ordinal is four digits. Reproduced against `origin/main`:

```
lexicographic order must be accepted by the strict-sort gate:
  "supersedes edges must be strictly sorted by numeric sequence and full predecessor ID"
```

`approved_scope` produces exactly that order. The CI harness already carries a `CHG-10000`
fixture (`.github/scripts/test-classify-ci-paths.sh:15`).

## Digest exposure: none, measured

Both the supersedes list and the succession tuples are serialized into digest preimages, so
reordering them is not free in principle. Measured across the whole archive:

```
archived records:                    160
with any supersedes edge:              0
with >=2 supersedes edges:             0

verification.json files:             160
with semantic_succession evidence:     0
with >=2 tuples:                       0
```

The succession subsystem has never been exercised in this repository's own history, so no
historical digest can move. The CHG-0068 golden vector remains the standing check that none did.

## What the happens-before guard is actually for

`:7519` sits between two checks that already establish the workflow property:

- `:7511` — the predecessor must be `Accepted | Archived`, i.e. complete.
- `:7525` — `supersedes_reaches` is a real cycle check over the edge graph.

A successor declares `supersede` *before definition approval*, so it is in flight while the
predecessor is complete; ordering is implied. And a backwards edge cannot be declared honestly
at all, because `load_change(root, &edge.predecessor_id)?` on the line above would fail on a
predecessor that did not yet exist.

So the guard's residual job is resistance to a hand-edited `supersedes` edge — not workflow
enforcement. `created_at` serves that exactly as well as an ordinal (both are fields in the same
file an attacker would be editing) while meaning the right thing.
