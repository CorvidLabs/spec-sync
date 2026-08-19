---
change: CHG-0157-a-newer-six-must-be-readable-by-an-older-six
artifact: testing
---

# Testing

## Discrimination

Against a binary built from a **separate clone** at `ad65908f`, the commit before this change:

| test | pre-valve | this change |
|---|---|---|
| `evidence_written_by_a_later_six_still_parses` | **FAILED** | ok |
| `an_unknown_workflow_version_says_upgrade_rather_than_invalid` | **FAILED** | ok |
| `regenerable_caches_still_reject_what_they_cannot_understand` | ok | ok |

The cache test passing on **both** is the control, and it is the one that matters: it is what
stops this change from being "delete every `deny_unknown_fields`". Tolerance is for evidence,
which cannot be recreated; a cache that cannot be understood must still be discarded.

## No digest moved

This is the claim with the most at stake — a changed preimage invalidates every historical
archive in every repository.

```
digest-bearing tests                    18 passed, 0 failed  (+1 integration)
CHG-0068 golden vector
  scope_adoption_fails_closed_when_anchor_is_unavailable_or_replayed   ok
```

The golden vector recomputes `scope_digest` from `tests/fixtures/chg-0068-adopted-scope.json`
and compares against a pinned constant, so it fails if the `ApprovedScopeV1` preimage shifts by
a byte. It does not.

The mechanism is why: `deny_unknown_fields` is a **deserialization** attribute. Serialization is
untouched, so no preimage can move.

## The limit, stated rather than glossed

`ApprovedScopeV1`, `CorrectionRecord` and `ScopedReviewRecord` are digest preimages. Adding a
field to one of those still changes its serialized bytes and therefore its digest. This change
lets an older reader **parse** such a file instead of erroring; it does not make field addition
digest-safe for those three. The other fourteen are freely extensible now.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-079 | Payloads carrying unknown fields deserialize into `ScopedReviewRecord`, `WorkflowV2Baseline` and `CorrectionLedger`, all of which rejected them at `ad65908f`; a record with `workflow_version: 9` now names a newer writer and an upgrade and no longer reads as an invalid change state, also failing at `ad65908f`; `HashCache` still rejects an unrecognised shape on both binaries, which is the control that keeps the change from being a blanket deletion; and the CHG-0068 golden vector plus 19 digest tests confirm no preimage moved |
