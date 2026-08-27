---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: requirements
---

# Requirements

This change modifies `REQ-change-089` and invariant 38 of `specs/change/change.spec.md`. It adds
no new requirement: the binding already existed, and its stated preimage — "the exact bytes" —
described something the code should not have been acting on.

## What the corrected requirement says

- The per-module digest is taken over the delta body with `\r\n` folded to `\n`, because delta
  application already treats line-ending style as not part of the content. A checkout that
  rewrote a delta's line endings without editing a character SHALL NOT invalidate the approval
  that signed it.
- No other difference is folded. Trailing whitespace, blank lines and a lone carriage return keep
  changing the digest, so the binding's equality stays strictly narrower than the applier's —
  `markdown_block_matches` also trims surrounding blank lines and horizontal whitespace, and the
  binding deliberately does not.
- Folding line endings moves no digest recorded before it, because a body containing no `\r\n`
  hashes exactly as it did.

Everything else `REQ-change-089` asserted is unchanged: the refusal still names every drifted
module and the remedy, it is still evaluated above the already-materialized short-circuit, an
approval carrying no binding still proceeds, and the field is still omitted from persisted JSON
when absent.

## Out of scope, deliberately

- The dirty-versus-clean asymmetry in `definition_artifact_snapshot` (recorded in `design.md`).
  It is a different axis, workflow-v1 only, and already guarded where it matters.
- Any widening of the normalization. See `design.md` for why that would be a regression rather
  than an improvement.
