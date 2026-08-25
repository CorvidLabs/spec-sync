---
change: fold-the-frontmatter-unification-lessons-into-the-parser-and-change-contexts
artifact: context
---

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
