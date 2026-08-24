---
change: fold-the-lessons-loop-bundle-into-the-change-cmd-change-and-generator-contexts
artifact: context
---

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
