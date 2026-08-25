# Lesson bundle — fold-the-delta-parser-lessons-into-the-change-context

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fold the delta-parser lessons into the change context
- **Kind**: Documentation
- **Paths**: specs/change/context.md
- **Acceptance**: the change context records the partial-fix lesson and the delta-digest coupling
- **Acceptance**: no canonical spec text or behaviour changes

## Evidence

- Verification commit: `4e379341b822762b7d1196ac551f9b3ccaf9858f`
- Base commit: `4e379341b822762b7d1196ac551f9b3ccaf9858f`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# context

Fold-back of the delta-parser lesson bundle into `specs/change/context.md`. Records that a partial fix disguises its own symptom, and that the duplicate-key guard is coupled to the flush ordering. No behaviour, requirement, or canonical spec text changes.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
