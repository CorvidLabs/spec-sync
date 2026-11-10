# Lesson bundle: generate-ownership-wording-matches-shipped-no-files-path

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Generate ownership wording matches shipped no-files path
- **Kind**: BugFix
- **Specs**: generator, cmd_scaffold
- **Paths**: src/generator.rs, specs/generator/, specs/cmd_scaffold/, tests/
- **Acceptance**: REQ-generator-009 and REQ-cmd-scaffold-006 describe the shipped behavior: an all-owned candidate set takes the existing no-files path (spec written with files: [] plus the empty-files warning); new foo against an owned flat file is covered by a test; the requirements carry no contradiction about config.modules mappings

## Evidence

- Verification commit: `194d566b155f3422d71d6d977db6e2eedef3755e`
- Base commit: `b2432e0488846b92f869e307c78b672354073415`
- Verified by: `specsync check --spec cmd_scaffold --spec generator`

## From the change's context.md

# Context






Round-1 review of #611 (PR #1025): the requirement text said "produces no spec" while the code
(and the pre-existing no-files path) writes the spec with files: []. This change rewords the
requirements to the shipped behavior and adds the missing `new` coverage. The archived round-1
workspace bullet about config.modules ("but generated specs still never double-map") contradicted
REQ-generator-009 AC3; the canonical wording stands: explicit mappings are the author's
declaration and are not filtered.

## From the change's testing.md

# Testing


Integration `new_never_claims_an_owned_flat_file` plus the round-1 scaffold tests.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-generator-009 | integration `new_never_claims_an_owned_flat_file`, `scaffold_never_claims_a_file_another_spec_owns`, `scaffold_single_source_fallback_skips_an_owned_file` |
| REQ-cmd-scaffold-006 | integration `scaffold_never_claims_a_file_another_spec_owns` |

## Where these lessons go

- `specs/generator/generator.3md`
- `specs/cmd_scaffold/cmd_scaffold.3md`
