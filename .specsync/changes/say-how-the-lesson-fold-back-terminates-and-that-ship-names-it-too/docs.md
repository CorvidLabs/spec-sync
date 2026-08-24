---
change: say-how-the-lesson-fold-back-terminates-and-that-ship-names-it-too
artifact: docs
---

# Docs

One file changes: `docs/ADOPTING.md`, section "Close the learning loop". No other doc, no spec
text, no source.

## Correction to the existing bullet

The third bullet read "At `finalize`, spec-sync writes a `lesson-bundle.md` into the archive and
names the step". Since #700 both `finalize` and `ship` prefix the fold-back onto their
`next_action`, so the bullet now names both verbs and says the clause precedes their remaining
guidance rather than being the whole of it — `ship --push --wait` continues with CI and merge
after it.

## Addition

Four short paragraphs and one command block after that bullet:

1. **The recursion, named.** The fold is itself a change touching tracked paths, so it needs its
   own record; declaring that record against the modules it edits reproduces the instruction.
   States plainly that there is no cycle detection and no warning.
2. **The termination.** Declaring no affected specs leaves nothing to fold into, so both verbs
   print their plain merge guidance. Followed by a copy-pasteable `change new` invocation:
   `--kind documentation`, a `specs/<module>/context.md` path, `--no-spec-change`, and rationale
   wording, so the one flag combination that terminates the loop is not improvised.
3. **Why the absent flag matters.** `--no-spec-change` alone is not sufficient, because
   ADOPTING.md already tells the reader that `--spec` and `--no-spec-change` coexist. The
   distinguishing fact is that a fold declares no `--spec` at all. Also states what the rationale
   must be true of, so it is not copied onto a change it does not describe.
4. **The boundary.** Keep a fold change to `context.md` paths. A spec companion is not
   production source and so does not trip the owning-module refusal documented earlier in the
   same page; production source in the same change does, and such a change has lessons of its own
   to fold. Fold separately.

Closes with the measured number — 6 of 183 archived changes have ever touched a spec's
`context.md` — in the same register as the page's other measurements, because the point is that
the fold-back is the step that gets skipped.

## Not documented here

The behavioural alternative from #703 (`finalize` recognising a companion-only change and
omitting the clause) is not described, promised, or hinted at. Until it exists, documenting it
would describe a tool the reader does not have.
