---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: docs
---

# Docs

No user-facing documentation page changes. The behaviour is internal to how SpecSync reads a
Markdown file's frontmatter; nothing in `docs/` or `site/` describes the delimiter rule, and no CLI
surface, flag, or output string changes.

What is documented, and where:

| Where | What it now says |
|-------|------------------|
| `specs/parser/parser.spec.md` | Invariants 16–18: the one delimiter rule, the half that must not loosen and the stated residual, and the LF body for a CRLF-only body. Two new Behavioral Examples and two new Error Cases rows. |
| `specs/change/change.spec.md` | Invariant 35: what artifact completeness now guarantees — including the BOM case #715 fixed without claiming — and the one residual it does not. |
| `specs/parser/context.md` | Why the line is drawn at trailing whitespace, why the reported hole had two ends, and that the rule is spelled twice with a test guarding the drift. |
| `specs/change/context.md` | Why the stripper's delimiter rule is an approval-gate question and not a cosmetic one, and why deriving the gate from the scaffold does not close the residual. |
| `specs/parser/tasks.md` | The residual, and the `commands/lifecycle.rs` sibling left unfixed. |
| `specs/parser/testing.md`, `specs/change/testing.md` | The new assertions and their honest labels. |
| `src/parser.rs` | `is_frontmatter_delimiter`'s doc comment carries the rule and its reasoning; `FRONTMATTER_RE`'s comment names the test that fails if the two spellings drift. |
