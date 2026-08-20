# Research

## What steps 1-4 already accomplished

On the unpatched binary, a hand-converted slug-only workspace passes the entire lifecycle. Three
surfaces fail, and all three through one line — `located_change_sequences`:

```rust
let sequence = change_sequence(&record.id)
    .ok_or_else(|| format!("invalid change ID `{}`", record.id.escape_default()))?;
```

Measured on the real 164-archive corpus with one slug-only workspace present:

| | before | after |
|---|---|---|
| `change audit --strict` | rc=1 `Auditing active changes (0)… invalid change ID` — cannot even count it | rc=0 `Auditing active changes (1)…` |
| `change new "<anything>"` | rc=1, same error — **creation dead repo-wide** | rc=0 |
| `change status` | rc=0, reports a **healthy next action** | reports the real state |

The third row is the trap. `sequence_ledger_freeze_next_action` pattern-matches known error
strings and ends `Err(_) => None`, so an unrecognised error yields a clean next-action: the
project looks fine and cannot create another change.

## The ledger is load-bearing twice over

`.specsync/change-sequence.json` cannot be deleted. Two independent reasons, and the first is
not the one that was assumed:

1. **`acknowledged_collisions` lives only in that file.** Five groups — ordinals 16, 48, 49, 99,
   100 — covering 11 archived changes. Delete the file and `validate_change_sequences` fails
   closed on unacknowledged duplicates before anything reaches the manifest layer.
2. **120 of 164 archives sign it** in their `verification.json` acceptance manifest, each having
   signed different content. Recomputing every entry against a pristine clone:

   ```
   exact live bytes:                     1   (the current owner)
   rescued by reconstruction:          113
   rescued by successor forgiveness:     6
   ```

   Those two mechanisms cover **disjoint** sets, so neither is redundant.

## Freezing is neutral, not an enabler

Advancing the ledger normally and re-running the census shows the current owner flipping from
`LIVE_BYTES` to `RECONSTRUCTED` and still passing; nothing else moves. Each archive signed
different content, so 119 stay on the reconstruction path forever whether the file moves or not.

Freezing is worth doing because it stops the population of archives-needing-reconstruction from
growing — not because it simplifies any archive that exists.

## Corrections to earlier assumptions

- **There is no cross-module compile error.** `src/commands/change.rs:2597` only breaks if
  `floor_sequence_ledger_to_committed` is also deleted. It is kept, and that file is untouched.
- **`SEQUENCE_PATH` has 31 references in non-test `src/`**, not 23 — the lower count missed 8
  hard-coded string literals.
- **The frozen ledger's owning archive can be deleted.** `validate_change_sequences`
  short-circuits when `ledger.sequence > maximum`, so the claim-present gate takes the escape
  path permanently under a freeze. It is inert, not a tamper detector.
