---
change: CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar
artifact: research
---

# Research

## Prior art in this repository

- **#626** established the rule this change reuses: a narrower signal may narrow a decision,
  never contradict a broader one. There it was a tip-only commit classification versus the whole
  pull request; here it is the ship lane versus the lifecycle state.
- **#536** is the same structural failure as layer (b): a fix landed where the report pointed
  while a parallel implementation survived. There the tolerance existed on one arm of a `match`
  and not the other; here `change_dir` existed and `commands/` grew its own copy anyway.
- **#629** is the precedent for what lenient reads must *not* become. It replaced "measured at
  zero" with an explicit `Withheld` state, because reporting an unmeasurable thing as a number
  is how absence becomes false reassurance. This change reports absent evidence as absent, never
  as satisfied.

## What was measured before designing

Two shims built against the unfixed tree, to find out whether the existing gate could judge a
fix at all:

- A one-line swap in the text printer flipped three of four gates and left drills 030 and 031
  byte-identical, because their `ship_next` controls read JSON while the swap lives in text.
- A three-line patch asserting `done` for an archived state, reading no evidence whatsoever,
  produced `8/0/0 PASS` — the board a complete fix is supposed to produce — on a binary still
  printing `Verification: none` and returning `verification_commit: null`.

That is why sandbox #90 landed first. A gate that a cosmetic patch can satisfy is not evidence,
and this change would otherwise have been "verified" by it.

## Enumeration

Every hard-coded active-workspace path under `src/commands/`:

| site | fate |
|---|---|
| `change.rs:772` verification | routed through the resolver |
| `change.rs:803` review | routed through the resolver |
| `change.rs:2209`, `:2248` | inside `#[cfg(test)] mod tests` (opens `:2067`) — fixtures, correctly active-only |

`find_change_dir` call sites were reviewed to confirm exposing it changes no existing behaviour:
it is already the active-or-archive answer everywhere it is used.

## Known limit, recorded rather than fixed

The "an archived change has at most one `[current]` stage" property holds **pre-merge only**. In
a squash-merge fixture the verification commit object survives but `merge-base --is-ancestor`
fails, so `verified` goes false and `product_tip` returns to `[current]`. Since essentially every
archived change a user inspects lives on a squash-merged `main`, that property should not be read
as a general invariant. Fixing it means reconsidering what "verified" means after a squash, which
is a larger question than this change.
