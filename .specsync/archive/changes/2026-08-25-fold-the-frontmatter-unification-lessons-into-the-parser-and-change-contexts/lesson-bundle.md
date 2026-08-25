# Lesson bundle — fold-the-frontmatter-unification-lessons-into-the-parser-and-change-contexts

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fold the frontmatter-unification lessons into the parser and change contexts
- **Kind**: Documentation
- **Paths**: specs/parser/context.md, specs/change/context.md
- **Acceptance**: the parser context records that five readers of one format disagree in different directions and that exactness cuts both ways
- **Acceptance**: the change context records that two of its own lessons were wrong and why a wrong lesson is load-bearing
- **Acceptance**: no canonical spec text or behaviour changes

## Evidence

- Verification commit: `35027aafdcc71721fed277bcd6cc8535ebe47d28`
- Base commit: `35027aafdcc71721fed277bcd6cc8535ebe47d28`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

Fold-back for the frontmatter unification, from its archived bundle.

`next_action` named three targets: `specs/parser/context.md`, `specs/view/context.md`, and
`specs/change/context.md`. Only two are folded here, deliberately. The view context's recorded
lesson pointed readers at a stripper that the unification deleted, and that correction was made
inside the unification change itself — it had to be, because leaving it would have shipped a
context file describing code the same change removed. There is nothing further to fold there.

The `change` context entry is unusual: it records that two lessons already in that file were
WRONG, and were corrected by the change whose bundle this is. Both were mine — asserted from a
grep count rather than read from the call sites, and folded before the correction landed. That is
recorded rather than quietly edited, because a reader who saw the original claim should be able to
see it refuted rather than find it silently absent (#714).

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
