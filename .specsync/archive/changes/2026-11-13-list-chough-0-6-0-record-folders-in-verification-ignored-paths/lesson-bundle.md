# Lesson bundle: list-chough-0-6-0-record-folders-in-verification-ignored-paths

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: List Chough 0.6.0 record folders in verification_ignored_paths
- **Kind**: Operations
- **Paths**: .specsync/sdd.json
- **Acceptance**: .specsync/sdd.json verification_ignored_paths lists .chough/sessions/, .chough/archive/ and .chough/bugs/; chough doctor reports no records_in_verification finding; specsync check passes

## Evidence

- Verification commit: `ea81f0ede696bb4fb8e185cbd6f2a49012f9f711`
- Base commit: `72eb5123fa45ddf3144d16d188cc1c6acb1b50ac`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Chough 0.6.0 adoption (2026-11-12). After `chough migrate`, `chough doctor` and the
migration receipt report `records_in_verification`: `.specsync/sdd.json` has no
`verification_ignored_paths` entry for Chough's record folders, so a Chough record
written after `specsync change check --commit` would stale the recorded verification.
Chough never writes this policy itself; its docs say adding the entries is an ordinary
reviewed change to `.specsync/sdd.json`. Leif's adoption instructions name this
follow-up as a small SpecSync change.

`.specsync/sdd.json` is both a meaningful path and a strict path in this repository, so
the change needs its own native lifecycle (scope approval, check, review, finalize).

## From the change's testing.md

# Testing

- `specsync check` passes with the edited policy.
- `chough doctor` reports no `records_in_verification` finding.
- `strict_verification_commands` for `.specsync/sdd.json` (strict path) run during
  `specsync change check`.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
