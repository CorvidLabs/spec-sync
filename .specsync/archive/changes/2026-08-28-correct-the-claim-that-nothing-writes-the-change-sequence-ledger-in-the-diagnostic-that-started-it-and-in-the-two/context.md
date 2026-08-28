---
change: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-two
artifact: context
---

# Context

## What led here

The #714 audit of folded lessons found it — the audit doing exactly what #728 part 2 is about,
except the false claim it caught had been folded from **me**.

`.specsync/change-sequence.json` was described as read-only, frozen, or never written in three live
files. It is written. `floor_sequence_ledger_to_committed` (`src/change.rs:1869`) calls
`write_json` on that path and runs on the commit path from `src/commands/change.rs:2865`, so every
lifecycle commit can rewrite it. That write is #533's fix: it **raises** a stale working-tree ledger
to the committed high-water mark and merges `acknowledged_collisions`.

The true statement is narrower: **nothing *allocates* into the ledger any more** — #665 retired the
ordinal — **but it is still written, in one direction, as a repair.**

## The origin, and why the copies are one change

I filed #728 arguing a requirement describing deleted behaviour is invisible to the drift check, and
quoted the diagnostic at `src/change.rs:2189` as evidence without reading the forty lines above it.
From there the claim was copied into #732's `AGENTS.md` rewrite, into the `CHANGELOG.md` entry for
#665, and into the public roadmap (Discussion #339, corrected separately and stated rather than
silently edited).

So this is one error and its copies, not three errors.

## The lesson worth folding

**An error message is prose, and it is the most trusted prose in a codebase.** It is written by the
authors, printed by the running program, and read by someone already in trouble looking for ground
truth. I trusted `src/change.rs:2189` *because* the tool emits it — the same reflex that makes a
folded lesson dangerous (#714): the reader has no reason to doubt it and no diff to review it
against.

Nothing tests a diagnostic string for truth. Requirements have a drift check pointed at them, however
imperfect. Doc comments at least sit beside the code they describe. A diagnostic can be arbitrarily
far from the function that falsifies it — here, 320 lines — and the further it drifts the more
authoritative it sounds.

## The second lesson, learned on this change

The first attempt was scoped `--path src/change.rs` with `--no-spec-change` and **no `--spec`**. It
passed `check`, passed independent review, and was refused at `ship`:

    acceptance input `src/change.rs` is production source without deterministic canonical ownership

That is REQ-change-033, and it is the trap `docs/ADOPTING.md` documents in as many words: the flag
means *no spec text changes*, not *no module owns this*. The refusal arrives several stages after the
only place it could have been fixed, and scope freezes at approval, so the whole change had to be
redone. **`--spec` and `--no-spec-change` are not alternatives; they answer different questions and
compose.** This change declares `--spec change` for ownership and `--no-spec-change` because no
canonical spec text moves.

## Ruled out

- **Touching `specs/change/context.md`**, which carries the same false claim. The #714 audit owns
  that file in a concurrent change. Two changes correcting one line is how a merge keeps one and
  silently drops the other — the failure #733's rebase caught in `change.spec.md`, where git merged
  two identical version increments into one and swallowed a version.
- **Rewriting the diagnostic to describe the writer.** Its reader is repairing a ledger, not
  learning the architecture. It now says only what they can act on.
- **Asserting the absence of a writer in a test.** Not expressible, and the claim was false anyway.

## Left for someone else

`src/change.rs:1802-1806` — a doc comment still describing the deleted allocation model, directly
above the function that falsifies it. Found by the #714 audit, recorded there, out of scope here.
