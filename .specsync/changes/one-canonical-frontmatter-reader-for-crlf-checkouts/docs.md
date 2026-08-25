---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: docs
---

# Docs

No end-user documentation changes. Nothing in `site/src/content/docs/`, `README.md`, `AGENTS.md`
or `docs/ADOPTING.md` describes frontmatter line-ending handling, and nothing needs to start: the
change removes a failure rather than adding a surface. A Windows user's experience goes from
"`specsync view` reports 'Cannot parse frontmatter' for every spec" to "it works", which is not a
documented behaviour to update.

The contract is documented where it is enforced:

- `specs/parser/parser.spec.md` — invariants 1, 13, 14 and 15, the `strip_frontmatter` Public API
  row, two new Behavioral Examples, and Error Cases rows stating that CRLF is not a failure
  condition.
- `specs/view/view.spec.md` — invariants 8 and 9, saying `view` owns no stripper and depends on
  the parser's CRLF tolerance rather than re-implementing it.
- `specs/change/change.spec.md` — invariant 35 rewritten to name the canonical reader, and
  invariant 38 for the `.gitattributes` pin.
- `.gitattributes` — the pin carries its own comment, matching the one already above the JSON
  patterns.

Three companion `context.md` files carried statements this change makes false and are corrected
with it rather than at fold-back time, because a lesson that is no longer true is worse than no
lesson: `specs/change/context.md` claimed "only `parser.rs` handles CRLF" and that its own
stripper's divergence from the repository convention was deliberate; `specs/view/context.md`
pointed a reader at a `strip_frontmatter` that no longer exists in that module.
