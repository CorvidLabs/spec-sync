---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: requirements
---

# Requirements

Adds `REQ-change-092` to `specs/change/requirements.md` and Invariant 41 to
`specs/change/change.spec.md`.

`REQ-change-092` — Canonical materialization SHALL be decided from the canonical artefacts rather
than from the `canonical_applied` flag alone, so that every output materialization produces is
present for the delta bodies the change has currently approved.

The five acceptance criteria carried in the delta, and why each is normative rather than
descriptive:

1. **A corrected, re-approved delta reaches the canonical spec on the next `check`.** This is the
   filed defect. The purpose of correcting a delta after review is to change the canonical spec,
   and this was the one path where doing what review asked for discarded the result in silence.
2. **A spec carrying the change's contract text but no version bump and no Change Log row receives
   both.** The widening. Neither output is derivable from a delta digest, which is what rules out
   the narrowest repair; stating them here keeps a future fix from satisfying only the first
   criterion.
3. **A byte-identical re-approval writes nothing.** The boundary. Without it "always
   re-materialize" satisfies criteria 1 and 2 and destroys the reason the short-circuit exists.
4. **Re-materialization does not refuse the work its own earlier run performed, and a FIRST
   materialization still fires every application refusal.** Both halves are normative: the first
   keeps a corrected `## REMOVED` delta from becoming a hard error, the second keeps the applier's
   real refusals — a delta naming a block that never existed — from being laundered into a no-op.
5. **The refusal for a drifted delta names `check` as well as `approve`.** An error message is
   prose too (#739). A remedy naming only half the sequence walked the author into the defect
   being reported.

Existing requirements this change does not weaken: `REQ-change-059` (content loss through the
`canonical_applied` path) and Invariants 38 and 40 (the delta binding and its monotonicity) are
untouched — the binding still refuses a delta that changed after the approval that signed it, and
this change adds the question that binding never asked.
