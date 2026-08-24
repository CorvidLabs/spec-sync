---
change: frontmatter-stripping-and-scaffold-detection-must-survive-crlf-and-an-unexpanded-module-placeholder
artifact: context
---

# Context

Found by end-to-end sandbox testing of the lessons loop in a genuinely new project, not by the
unit suite — every unit fixture was LF.

A byte-identical generated scaffold, differing only in line endings, behaved differently:

    LF   pristine scaffold  -> silent, correct
    CRLF pristine scaffold  -> "specs/winmod/context.md (1 line(s)) — read before scoping"

So a Windows-authored project is told every untouched module has recorded knowledge. That is the
precise failure the lessons loop exists to prevent: a pointer to nothing trains the reader to
ignore the pointer, and it lands on new adopters, who are the people stage 1 is for.

## Two causes stacked, either silent alone

1. `strip_frontmatter` matched literal `---\n`. `parser.rs` accepts `---\r\n`; this did not, so
   CRLF frontmatter survived into content counting (#696).
2. `is_generated_context_line` compared against the RAW template, which carries the unexpanded
   `{module}` placeholder. A real file says `spec: winmod.spec.md`; the template says
   `spec: {module}.spec.md`. They can never be equal.

Cause 2 was invisible on LF because frontmatter was stripped before it could matter. It only
appears when stripping fails. Fixing either alone leaves the other latent.

## Why the unit suite could not see it

`specs/generator/context.md` already carried the lesson, folded from #697's bundle: *a defect in
scaffold handling is invisible to dogfooding on this repository, because all 62 specs have
authored prose and no untouched scaffold exists here to trip over.* That lesson was read at
`change new` for this change and is why the fixtures below are the real generated artifact rather
than hand-written approximations.
