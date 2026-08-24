# Lesson bundle — fold-the-lessons-loop-bundle-into-the-change-cmd-change-and-generator-contexts

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fold the lessons-loop bundle into the change, cmd_change and generator contexts
- **Kind**: Documentation
- **Paths**: specs/change/context.md, specs/cmd_change/context.md, specs/generator/context.md
- **Acceptance**: each affected module context.md gains the durable lesson from the lessons-loop bundle, synthesised rather than restated from the change description
- **Acceptance**: no canonical spec text, requirement or behaviour changes
- **Acceptance**: the next change to these modules sees a higher substantive line count at change new, which is the loop compounding

## Evidence

- Verification commit: `bfd3ef6224b2b311890ad9299a5d9bb3c8e371a0`
- Base commit: `fb88b9acaafe99abd83a637876331e83330e49fb`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

The fold-back step `finalize` instructs, performed for the lessons-loop change (#697) from the
bundle it left at
`.specsync/archive/changes/2026-08-24-close-the-lessons-loop.../lesson-bundle.md`.

This is the step the whole loop exists to enable, and it had never been run for real. Doing it
surfaced its own friction: editing a module's `context.md` normally requires declaring that spec
module, which requires a semantic delta — even when no spec text changes. `--no-spec-change` is
the intended path and makes it straightforward, but it is not obvious, and only 4 of 178 archived
changes have ever touched a `context.md`. That number is the evidence that knowledge was being
written into changes and never reaching the specs.

The lessons themselves are synthesised, not restated: each module gets what it should know next
time, not a summary of what this change did.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
