# Lesson bundle: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change adopt --dry-run refuses an unsupported import source as adoption does
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, tests/integration/change.rs
- **Acceptance**: change adopt --dry-run --source bogus exits nonzero with unsupported adoption source and prints no import plan, exactly as change adopt --source bogus does; a dry run with a supported or detected source still prints its plan and writes nothing

## Evidence

- Verification commit: `830ee703ea658a310c0224c439be5a0f6d39941b`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#432 (filed against 5.2.0). Its main defect, adopt turning on path coverage in a local-only
repository, was confirmed fixed on 2026-11-10. The remaining gap: `adopt`
(`src/change/historical_definition_digest_matches.rs`) returned the dry-run plan before it
called `validate_foreign_import`, so `change adopt --dry-run --source bogus` exited 0 and printed
"import active and canonical artifacts from bogus", while `change adopt --source bogus` refused
with `unsupported adoption source`. The help documents `--dry-run` as a preview of adoption and
`--source` as `openspec` or `speckit`.

The fix runs `validate_foreign_import` in the dry-run branch before returning the plan. It only
reads metadata (symlink and file-type checks), so the dry run still writes nothing. The real run
keeps calling it under the project lock, unchanged.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.

## From the change's testing.md

# Testing

## Requirement evidence

| Contract | Evidence |
|---|---|
| `change adopt --dry-run` previews adoption; `--source` is openspec or speckit | `change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does` (red before the fix, green after) |
| A dry run writes nothing | the same test and the existing `adopt_dry_run_does_not_enable_policy` |

## How it runs

`change check` runs only the in-process spec-code sync. The tests run locally with
`cargo test --test integration adopt`, and in hosted CI's full suite at the archived head.

## Where these lessons go

- `specs/change/change.3md`
