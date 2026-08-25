---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: tasks
---

# Tasks

- [x] Normalize CRLF inside `parser::parse_frontmatter`, guarded on `content.contains('\r')`, after
      the BOM trim and before the regex; keep the returned `body` LF-only
- [x] Add `parser::strip_frontmatter`, promoted verbatim from `change::strip_frontmatter`, with its
      doc comment rewritten to state the six axes and the deliberate no-normalization asymmetry
- [x] Delete `change::strip_frontmatter`; import the canonical one
- [x] Delete `change::strip_yaml_frontmatter`; repoint `artifact_content_is_incomplete`
- [x] Delete `view::strip_frontmatter`; import the canonical one
- [x] Pin `.specsync/**/*.md text eol=lf` in `.gitattributes`
- [x] Parser tests: CRLF document, CRLF + BOM + body rule, the six-axis stripper case, plus the
      LF control, the lone-CR invariant and the no-frontmatter control
- [x] View tests: CRLF spec renders, CRLF companion is stripped, LF control
- [x] Change tests: CRLF artifact with an LF body rule is complete, frontmatter-only-at-EOF
      artifact is incomplete, LF verdict control
- [x] Verify every discriminator red against the pre-change implementation before accepting it
- [x] Semantic deltas for `parser`, `view` and `change`
- [x] Correct the now-false lessons in `specs/change/context.md` and `specs/view/context.md`, and
      record the new decision in `specs/parser/context.md`
- [x] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`

## Carried elsewhere, not left undone here

These are named so they are not mistaken for oversights. None is a task of this change.

- Normalizing `\r\n` in `delta_body_digests` — the remaining half of #709. The function does not
  exist on `main`; it arrives with PR #711 (#704), and the work belongs on that PR. Argument and
  evidence recorded in `design.md`.
- `commands/lifecycle.rs:26`'s unanchored `find("---\n")`, which can edit a `status:` line in a
  document BODY. Step 4 of the #696 migration order; a real, orthogonal bug in another module.
- A source-grep test forbidding new `strip_prefix("---` outside `parser.rs`. Step 6 of the same
  order.
