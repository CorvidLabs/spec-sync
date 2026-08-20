# Design

## The ordinal claim, narrowed

`change_sequence` returns `None` for two different situations and the old code conflated them.
`located_change_ordinal` separates them:

| ID | claims an ordinal? | result |
|---|---|---|
| `a-slug-only-change` | no | `Ok(None)` — absent from numeric accounting |
| `CHG-abcd-malformed` | no (not digits) | `Ok(None)` |
| `CHG-123-too-short` | **yes**, badly | `Err` — fails closed |
| `CHG-09999-noncanonical-width` | **yes**, badly | `Err` — fails closed |

Dropping the malformed ones silently would take them out of the acknowledged-collision ID-set
check that guards the archived collision members. A blanket `continue` — the shape of an earlier
attempt at this fix — is a small regression riding along with a real one, so it was not taken.

## The duplicate-identity gate, made explicit

The ordinal made two changes sharing an identity impossible by construction, and the numeric
gate caught it as a side effect. A slug is unique only by convention, and two clones can archive
the same slug on different days into differently-dated directories that git merges without a
conflict.

So the gate is now explicit and does not go through the ordinal, because the IDs that need it no
longer have one. It lives in `validate_change_sequences` rather than in
`list_all_changes_uncached` — which already refuses the same shape — because `change audit` runs
with `include_archive_integrity = false` and never loads the archive at all.

## Allocation

`allocate_change_workspace` escaped a taken directory by incrementing the ordinal. With no
ordinal that loop retries the same path 10,000 times and reports *"exhausted change sequence
allocation retries"* for what is really a repeated description. It now names the existing change:

```
error: a change named `retire-the-ordinal-at-last` already exists
  .specsync/changes/retire-the-ordinal-at-last
  it is: retire the ordinal at last (draft)
```

## What survives, and why it looks dead

Roughly 400 lines of history-reading machinery stay: `load_change_sequence_ledger`,
`historical_sequence_ledger_acceptance_content`, `later_sequence_owner_covers_historical_input`,
the collision accounting, and `floor_sequence_ledger_to_committed`. They exist for the 120
archives that signed the ledger and will never stop being needed.

Net **−89 lines of production code and +52 lines of comment**, because code that only serves
history has to say so or the next person deletes it.

## Regression, measured

One representative per risk class against a baseline captured before any of this work:

| class | archives | expected | actual |
|---|---|---|---|
| legacy baseline | 44 | authenticated-history | authenticated-history |
| stage A+B | 19 | authenticated-history | authenticated-history |
| stage B only | 90 | authenticated-history | authenticated-history |
| pre-existing corrupt | 7 | corrupt-history | corrupt-history |

**0 of 164 archives change validity.**
