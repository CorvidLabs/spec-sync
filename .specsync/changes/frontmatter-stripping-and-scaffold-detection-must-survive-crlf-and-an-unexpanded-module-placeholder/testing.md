---
change: frontmatter-stripping-and-scaffold-detection-must-survive-crlf-and-an-unexpanded-module-placeholder
artifact: testing
---

# Testing

Fixtures are `generated_context_scaffold(module)` itself, converted to CRLF — not hand-written
lookalikes. A fixture the product cannot produce proves nothing about the product; that is the
mistake an earlier version of the LF test made.

- `accumulated_lessons_ignores_a_crlf_generated_scaffold` — the regression.
- `accumulated_lessons_counts_authored_crlf_prose` — the complement. Before the fix, counting
  worked on CRLF while suppression did not; they must agree.
- `generated_scaffold_expands_the_module_placeholder` — pins cause 2 on its own, so the expansion
  cannot be dropped while the stripper masks it.
- `strip_frontmatter_removes_crlf_frontmatter_and_keeps_later_rules` — CRLF stripping without
  reintroducing the horizontal-rule truncation.

## Discrimination

Demonstrated on the SHIPPED binary before the fix: a real CRLF scaffold in a fresh project printed
`(1 line(s))`. After the fix, the same project is silent. That is the discriminator, measured
against a separately built binary rather than asserted.
