---
change: CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete
artifact: docs
---

# Docs

User-visible message changes only; no CLI surface, flag, or file format changes.

- Evidence gaps now fail with `verification cannot start: …` and name
  `.specsync/changes/<ID>/testing.md` plus its `## Requirement evidence` table.
- Command failures name the failing command and exit code instead of reporting that
  "a configured verification command failed".
- Duplicate ordinals fail with the two claiming change IDs, the shared base commit, and the
  instruction to recreate one with `specsync change new`.

No README, MIGRATION, or site page describes the previous messages, so no doc edits are required.
