---
change: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-three
artifact: context
---

# Context

## What led here

The #714 audit of folded lessons found it, which is the audit doing exactly what #728 part 2 is
about — except the lesson it caught had been folded from **me**.

I filed #728 arguing that a requirement describing deleted behaviour is invisible to the drift
check. As evidence I quoted the diagnostic at `src/change.rs:2189` — *"nothing writes this file any
more"* — without reading the forty lines above it. From there the claim went into #732's
`AGENTS.md` rewrite, into the `CHANGELOG.md` entry for #665, and into the public roadmap
(Discussion #339). All three are now corrected; the roadmap correction is stated rather than
silently applied.

## The lesson worth folding

**An error message is prose, and it is the most trusted prose in a codebase.** It is written by the
authors, printed by the running program, and read by someone who is already in trouble and looking
for ground truth. I treated `src/change.rs:2189` as authoritative *precisely because* the tool
emits it — the same reflex that makes a folded lesson dangerous (#714): the reader has no reason to
doubt it and no diff to review it against.

Nothing tests a diagnostic string for truth. Requirements have a drift check pointed at them, however
imperfect. Doc comments at least sit beside the code they describe. A diagnostic can be arbitrarily
far from the function that falsifies it — here, 320 lines — and the further it drifts the more
confident it sounds.

## Ruled out

- **Making absence of a writer enforceable.** There is no way to assert "nothing writes this path"
  in Rust, and the claim was false anyway, so the fix is to stop making it.
- **Touching `specs/change/context.md`.** The #714 audit owns that file in a concurrent change.
  Two changes correcting the same line is how a merge silently keeps one and drops the other —
  which is the failure #733's rebase caught in `change.spec.md`'s version bump, where git merged
  two identical increments into one and swallowed a version.
- **Rewriting the diagnostic to describe the writer.** The reader of that error is repairing a
  ledger, not learning the architecture. It now says only what they can act on.

## Left for someone else

`src/change.rs:1802-1806` — a doc comment still describing the deleted allocation model, sitting
directly above the function that falsifies it. Found by the #714 audit and recorded there; out of
this change's scope, and not urgent, but it is the next copy of the same error.
