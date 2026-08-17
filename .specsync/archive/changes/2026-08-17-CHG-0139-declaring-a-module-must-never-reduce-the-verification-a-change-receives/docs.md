---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: docs
---

# Docs

A CHANGELOG entry lands under `## [Unreleased]` → `### Fixed`.

## What it must state, and does

- **The observable defect:** `change check` printed `✓ verified` and recorded `passed: true`
  while running none of the four configured verification commands.
- **The inversion**, because it is the part that is hard to believe and easy to verify: the change
  declaring two real modules ran one filtered command; the change declaring none ran all four.
  Declaring scope accurately cost coverage.
- **The mechanism in one line:** `commands.is_empty()` was evaluated per change rather than per
  module, so one routed module suppressed the project list for every module in scope.
- **That it contradicted a requirement already on the books** — `REQ-change-015`.

## Breaking-change disclosure

**`change check` gets slower for most changes, and that is the fix working.** 49 of this
repository's 62 spec modules have no routing entry, so any change naming one of them now runs the
full `cargo test` plus both release validators where it previously ran a filtered subset. Stated
plainly so it is not met as a regression.

## Stated limits

- **The ten archived records that carry narrowed evidence are not repaired.** The fix is
  forward-looking. Deciding whether to annotate or re-verify them is separate.
- **Zero-match is still undetected.** A `cargo test` filter selecting no tests exits 0 and is
  indistinguishable from a filter that matched and passed. Catching it requires capturing output,
  which `REQ-change-058` forbids. Tracked on #617.
- **The three routing keys remain undocumented and schema-unvalidated.** This change fixes their
  behaviour, not their discoverability.
