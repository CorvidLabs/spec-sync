---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: docs
---

# Docs

No user-facing documentation changes. There is no new flag, command, output shape or persisted
field: the change alters what one digest preimage contains, and the only externally visible effect
is that a refusal which should never have happened no longer happens.

Three internal documents did carry the wording that is now wrong, and all three are corrected:

- `specs/change/change.spec.md` invariant 38 — said "a digest over each delta file's exact bytes".
- `specs/change/requirements.md`, `REQ-change-089` — same phrase in its first acceptance criterion.
- `specs/change/context.md` — "one digest per module over the delta file's exact bytes", plus a
  new section recording why the line-ending axis is the only one folded, why a lone carriage
  return is kept, and what the sibling sweep found. That file is what `change new` shows an author
  before they scope anything, so getting it right is worth more here than in the other two.

`.gitattributes` already documents the LF pins from #715 and needs nothing further. Its own
comment already states the rule this change makes true in the reader as well as in the working
tree: an adopter's repository, a tarball, or an archive extracted without Git is never covered by
a pin.
