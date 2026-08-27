---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: context
---

# Context

Issue #714 found two false statements in `specs/change/context.md`, both folded in by the lessons
loop working exactly as designed, and closed with one sentence that scoped this change:
**"Nobody has audited the rest."**

A wrong lesson is not an ordinary wrong comment. `change new` puts a module's `context.md` in front
of an author BEFORE they scope anything, which is the whole point of #697. Knowledge arriving with
the authority of recorded experience, at the moment decisions are made, is load-bearing — and it is
harder to dislodge than a wrong code comment, because the reader has no diff to review it against.
The loop has no notion of truth, only of provenance.

Sixty-two `context.md` files, 2,549 lines, most of it older than the two corrections. This change
reads every checkable claim in all of them against the source and fixes the ones that are wrong.

## What made a claim checkable

A file or symbol exists; a named test exists; a count; a convention ("all N call sites do X"); a
pointer to a module or line; a behaviour settled by reading one function. Everything else —
judgement, rationale, the narrative of a past decision — is legitimate prose and was left alone.
Most of these files are mostly that, which is why 34 of 62 needed an edit and 28 did not.

## Constraints this worked under

- **Verify, never assess.** Every verdict here came from running a command or reading the source.
  Plausibility is what produced the original defect.
- **Recount every number.** The two known-false lessons died because a count was carried over with
  a different denominator than the reader would assume. No number was inherited, including the
  corrected `21 of 39` from #714.
- **Do not over-correct.** A claim that is imprecise but sound in context stays. Turning a
  true-but-loose statement into a differently wrong one is the same failure with the sign flipped.
  Several claims were examined and deliberately left: see `research.md`.
- **Fix the lesson, not the loop.** #714 offers four designs (cite evidence not conclusions; date
  and attribute; re-derive on read; correct loudly). All four are out of scope. This change
  corrects wrong statements and implements none of them.

## Prior attempts and what is already ruled out

- #696 corrected the two `specs/change/context.md` lessons #714 names. Both were folded here before
  the correction landed; the loop propagated the pre-correction version faithfully.
- #732 swept the `CHG-NNNN` allocation vocabulary out of `specs/change/change.spec.md` and two
  `context.md` paragraphs. It caught the allocation wording — and left `Nothing writes it any more`
  standing, which is false for a different reason (see `research.md`).
- #733 unified the three frontmatter readers behind one delimiter rule. Every frontmatter claim
  here was re-checked against the merged state, not against the state #714 described.

## Scoping note worth carrying

This change declares its 34 owning specs AND `--no-spec-change`. Declaring `--spec` alone makes
`validate_delta_files` require exactly one semantic delta per affected spec, and a delta rewrites
the canonical `<module>.spec.md` — which this change does not touch. `--no-spec-change` short-
circuits that check while `--spec` keeps ownership explicit, which is the combination
`docs/ADOPTING.md` documents for "the modules own these paths, but no canonical spec text moves".
