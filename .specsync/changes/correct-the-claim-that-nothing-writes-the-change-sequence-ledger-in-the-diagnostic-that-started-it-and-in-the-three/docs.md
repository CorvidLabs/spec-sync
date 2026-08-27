---
change: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-three
artifact: docs
---

# Docs

## The false claim

`.specsync/change-sequence.json` was described in three live files as read-only, frozen, or never
written. It is written. `floor_sequence_ledger_to_committed` (`src/change.rs:1869`) calls
`write_json` on that path, and it runs on the commit path from `src/commands/change.rs:2865`, so
every lifecycle commit can rewrite it. That write is #533's fix: it **raises** a stale working-tree
ledger to the committed high-water mark before staging, and merges `acknowledged_collisions`.

The true statement is narrower than what was published: **nothing *allocates* into the ledger any
more** — #665 retired the ordinal, and identity is now a slug — **but it is still written, in one
direction, as a repair.**

## The four sites, and why they are one change

They are not four independent errors. They are one error and three copies of it.

1. **`src/change.rs:2189`** — the origin. The diagnostic read *"nothing writes this file any more,
   so it cannot be repaired by allocating."* The subordinate clause is true and useful; the
   parenthetical assertion in front of it is false. It now reads *"nothing allocates a sequence any
   more, so this cannot be repaired by minting a higher one"*, which says the same operative thing
   without the false premise.
2. **`AGENTS.md:51`** — copied from the diagnostic, and self-contradicting inside one bullet:
   *"Nothing writes it"*, followed immediately by *"`change check` raises a stale working-tree copy
   to the committed mark and says so."* Now names `floor_sequence_ledger_to_committed` and the one
   direction the ledger may move.
3. **`CHANGELOG.md`**, in the `### Changed` entry for #665 — *"is frozen — nothing writes it any
   more."* Corrected, and the entry says outright that it carried the wrong claim until it was
   measured, because a changelog that silently revises itself is worth less than one that does not.
4. `specs/change/context.md:9` carries the same claim and is **deliberately not touched here** — it
   is being corrected by the #714 lessons audit, which owns that file. Fixing it in two changes at
   once is how a merge silently drops one of them.

The archived records under `.specsync/archive/` also contain the claim. They are immutable evidence
of what was believed at the time and are correctly left alone.

## Scope

No behaviour changes. One diagnostic string and two prose files. `cargo clippy -- -D warnings` and
the full suite are unaffected; no test pinned the old wording, which is checked and is part of why
it survived.
