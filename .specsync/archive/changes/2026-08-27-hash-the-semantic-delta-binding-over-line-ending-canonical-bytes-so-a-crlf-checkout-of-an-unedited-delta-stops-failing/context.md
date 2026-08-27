---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: context
---

# Context

#709 named two remedies. #715 landed the first — the `.gitattributes` `eol=lf` pins for
`.specsync/**/*.md` and `specs/**/*.md` — and said in as many words that the second had not
landed and someone still had to apply it. #711 then shipped the gate that needs it. This change
is that second remedy, filed as #730.

`delta_body_digests` hashed the delta body as raw bytes: `read_bounded_change_text` is a bounded
`fs::read_to_string` that normalizes nothing. But the code that APPLIES a delta treats
line-ending style as explicitly not part of the content, in three independent places:

- `markdown_block_matches` — its doc comment says it compares "ignoring line-ending style and
  surrounding blank lines", and it folds CRLF before comparing;
- `apply_markdown_block` re-emits every body in the target file's own style, trimming trailing
  terminators and re-expanding LF into the file's own ending;
- `parse_delta` reads through `str::lines()`, which discards the carriage return of a CRLF pair,
  so a CRLF delta and an LF delta produce byte-identical canonical specs.

So the module had a definition of when two delta bodies are the same, and the digest binding a
delta to the approval that signed it did not use it. A change approved on Linux and checked out
on Windows with `core.autocrlf=true` produced a different digest with nothing edited, and
`ensure_approved_delta_bodies_unchanged` refused honest work. The remedy that refusal names —
re-approve — re-signs bytes the operator did not choose and diverges again on the next handoff
back. A gate that refuses honest work is worse than an absent gate, because operators learn to
route around it.

## Why this repository could not see it

#715's pins keep spec-sync's own deltas LF on every platform, so the defect is invisible here and
live for every adopter without those pins. We fixed our own instance in #715 and shipped the
class. It also sits inside the guarantee 6.0 explicitly kept when it dropped the Windows binary:
the `### Removed` CHANGELOG entry states the retained case as "a teammate on Windows commits CRLF
files and a colleague on Linux reads them".

## Constraints that shaped the fix

1. **No recorded digest may move.** Measured rather than argued: all 198 archived
   `approvals.json` under `.specsync/` were recomputed under both the raw and the normalizing
   digest. 8 ledgers carry `approved_delta_digests`, 25 module records in total, and the two
   digests are identical for every one of them — 0 move. (Two of the 25 match neither recomputation,
   because they are superseded earlier definition approvals in #711's own ledger, whose delta was
   re-approved twice during that PR; the effective approval matches exactly, and those two differ
   identically under both digests, so this change does not move them either.)
2. **Do not widen beyond line endings.** `markdown_block_matches` also trims surrounding blank
   lines and horizontal whitespace. The digest must NOT copy that half: trailing whitespace and
   blank lines are wording a reviewer signed, and folding them would make the gate accept edits it
   exists to refuse. Only the line-ending axis is provably not content, because it is the only one
   Git rewrites with no author behind it.

## A lone carriage return, decided rather than omitted

Kept as content. Git's `text`, `eol` and `core.autocrlf` conversions only ever move between LF and
CRLF, so no checkout can introduce a classic-Mac terminator; `str::lines()` and
`markdown_block_matches` both keep a bare carriage return as ordinary text, so it reaches the
canonical spec, which makes it wording; and `parser::parse_frontmatter` preserves it deliberately
for the same reason (#715). A body that gained one was edited by a person.

## Sibling sweep

`delta_body_digests` was the only digest in the codebase framing filesystem text. Recorded in
`design.md`; nothing else needs the same change.
