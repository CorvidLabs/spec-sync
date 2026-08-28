# Lesson bundle — correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-two

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Correct the claim that nothing writes the change sequence ledger, in the diagnostic that started it and in the two places it was copied to
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, AGENTS.md, CHANGELOG.md
- **Acceptance**: No live file in the repository claims that nothing writes .specsync/change-sequence.json or that it is read-only or frozen; the diagnostic at src/change.rs states the true and useful constraint (nothing allocates, so it cannot be repaired by minting a higher sequence); and AGENTS.md and CHANGELOG.md name floor_sequence_ledger_to_committed as the one writer and the single direction it may move the ledger.

## Evidence

- Verification commit: `12c2ff6e6300b93554ce7eeba58f9499f69cbf49`
- Base commit: `4b72b09de0e950b7a0479463dbefcac33d516cac`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's testing.md

# Testing

## What is and is not verifiable here

No behaviour changes. One diagnostic string and two prose files.

**No test is added, deliberately.** The only assertion available would pin the new wording of an
error message, which would fail against unfixed `main` — satisfying the letter of the discrimination
protocol while proving nothing about correctness. A string-equality test on a diagnostic does not
establish that the diagnostic is *true*; it establishes that nobody has edited it. That is precisely
the false comfort this change is about, and adding one would be the defect wearing a test's clothes.

**Honest label: no DISCRIMINATOR exists for this change, and none should.** What would have caught
the original error is not a test but a reader checking a claim against the function that implements
it — which is what the #714 audit did.

## What was checked instead

| check | result |
|---|---|
| No live file claims the ledger is unwritten, read-only, or frozen | `grep -rn 'read-only history\|[Nn]othing writes'` over tracked files returns only `specs/change/context.md` (owned by the concurrent #714 audit) and archived records under `.specsync/archive/`, which are immutable evidence and correctly untouched |
| No test pinned the old wording | `grep -rn 'nothing writes this file\|repaired by allocating' src/ tests/` returns nothing but the source line itself — part of why the claim survived |
| The claim being corrected is actually false | `floor_sequence_ledger_to_committed` at `src/change.rs:1869` calls `write_json(&root.join(SEQUENCE_PATH), …)`; caller at `src/commands/change.rs:2865` is on the commit path |
| The replacement is true | Nothing calls an allocation path; #665 removed `maximum_observed_sequence` and `remote_sequence_high_water`, and #732 retired the last spec text describing them |

Full suite, `cargo fmt --check`, and `cargo clippy -- -D warnings` all run unchanged, since the
edit is inside a `format!` string literal and two Markdown files.

## Where these lessons go

- `specs/change/context.md`
