---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: docs
---

# Docs

## Canonical spec

- `specs/manifest/manifest.spec.md` — version 21 → 22. Invariant 14 now states that a balanced
  trailing configuration block is skipped rather than refused, and states the four things that keep
  it safe (path confined in front of the block, block text still under every other guard,
  quote-aware scan after comment stripping, unbalanced block refused). Error Cases gains an
  unbalanced-block row and a directives-inside-the-block row, and the two existing `includeBuild`
  rows say "with or without a trailing configuration block". Change log entry added.
- `specs/manifest/requirements.md` — the sentence that said trailing configuration blocks keep
  failing closed now says a balanced block is skipped, and a new acceptance criterion states that a
  block-carrying declaration reaches the same verdict as the same declaration without one.
- `specs/manifest/context.md` — new key decision "A configuration block is not a project
  declaration", and a `## Lesson (#725)` recording both halves: the parser fix drew its line one
  notch short of the reported form, and the precedence fix from #723 is what kept that from being a
  second outage.
- `specs/manifest/tasks.md`, `specs/manifest/testing.md` — completed task plus a behavioral
  verification row and a regression-matrix row for the block form.

## User-facing surfaces

No CLI flag, output field, config key, or exported symbol changes. The only observable difference is
that a Gradle project written the common way is now measured instead of producing
`Unsupported trailing Gradle includeBuild declaration expression`. No site or README page documents
the Gradle settings subset at that level of detail, so nothing there needs editing.
