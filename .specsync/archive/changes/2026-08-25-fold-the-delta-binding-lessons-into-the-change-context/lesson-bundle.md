# Lesson bundle — fold-the-delta-binding-lessons-into-the-change-context

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fold the delta-binding lessons into the change context
- **Kind**: Documentation
- **Paths**: specs/change/context.md
- **Acceptance**: the change context records that currency and completeness need different mechanisms
- **Acceptance**: it records the line-anchored comparison lesson
- **Acceptance**: no canonical spec text or behaviour changes

## Evidence

- Verification commit: `6479b5c18a72e2fbe3433cea71d5c9f17d1cdebb`
- Base commit: `6479b5c18a72e2fbe3433cea71d5c9f17d1cdebb`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# context

Fold-back of the delta-binding lesson bundle. Records that a digest proves currency but not completeness, and that block headings must be compared line-anchored. No behaviour, requirement, or canonical spec text changes.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
