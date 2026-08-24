# Lesson bundle — frontmatter-stripping-and-scaffold-detection-must-survive-crlf-and-an-unexpanded-module-placeholder

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Frontmatter stripping and scaffold detection must survive CRLF and an unexpanded module placeholder
- **Kind**: BugFix
- **Specs**: change, generator
- **Paths**: src/change.rs, src/change_tests.rs, src/generator.rs, specs/change/change.spec.md, specs/generator/generator.spec.md
- **Acceptance**: a pristine generated context companion with CRLF line endings is silent at change new, exactly as the LF one is
- **Acceptance**: CRLF authored prose is still counted, so suppression and counting agree on both encodings
- **Acceptance**: the generator hands out the EXPANDED scaffold, so the spec frontmatter line can match a real file instead of an unexpandable placeholder
- **Acceptance**: frontmatter is stripped at its closing delimiter line in either encoding, and a body horizontal rule still never truncates

## Evidence

- Verification commit: `ea9d8921c68241b193f7a38a7e92ac50fd10412d`
- Base commit: `9b6e03cd10d33d1278430b59b8a393d6d672e277`
- Verified by: `cargo test change::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's design.md

# design

CRLF-tolerant frontmatter stripping and an expanded scaffold comparison, so a Windows-authored context companion is treated exactly as an LF one.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/context.md`
- `specs/generator/context.md`
