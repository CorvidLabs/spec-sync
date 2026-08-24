# Lesson bundle — fold-the-crlf-scaffold-lessons-into-the-change-and-generator-contexts

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fold the CRLF scaffold lessons into the change and generator contexts
- **Kind**: Documentation
- **Paths**: specs/change/context.md, specs/generator/context.md
- **Acceptance**: the change and generator contexts record the durable lessons from the CRLF bundle, synthesised rather than restated
- **Acceptance**: no canonical spec text, requirement or behaviour changes
- **Acceptance**: the next change to these modules reads a higher substantive line count at change new

## Evidence

- Verification commit: `5aa60945c373e5feb70d2beb1217b5a144a2b485`
- Base commit: `ea9d8921c68241b193f7a38a7e92ac50fd10412d`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

The fold-back step `finalize` instructs, performed for the CRLF scaffold fix from the bundle it
left in the archive.

Two lessons were worth carrying, and neither is a restatement of what the change did.

**For `change`:** this module now diverges from the repository's normalize-then-parse convention.
That is a deliberate trade — keeping a borrowed `&str` return instead of allocating — but it means
the module owns a parser with its own CRLF dialect, and that its stripper is no longer
interchangeable with `view`'s. Both facts are the kind that decay silently; an earlier comment
claiming interchangeability was already false by the time anyone read it.

**For `generator`:** comparing against a raw template rather than an expanded one is a defect that
stays invisible behind an unrelated working step, and surfaces only when that step fails.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
