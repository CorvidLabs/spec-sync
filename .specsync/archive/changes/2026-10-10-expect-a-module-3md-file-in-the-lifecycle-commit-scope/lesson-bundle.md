# Lesson bundle — expect-a-module-3md-file-in-the-lifecycle-commit-scope

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Expect a module 3md file in the lifecycle commit scope
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change_tests.rs
- **Acceptance**: The lifecycle scope test includes specs/<module>/<module>.3md with the other companions.

## Evidence

- Verification commit: `5eda3df343d50ca8b7a685b7b4625ac357e31983`
- Base commit: `353765989a52957754cbefb909eaff9c6208d136`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Staging a new module.3md changed the lifecycle commit scope. The test that lists owned companions now includes that file.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-106 | `lifecycle_commit_scope_names_exactly_what_the_change_owns` passes with `specs/auth/auth.3md` in the owned set. The full `cargo test` suite passed locally. |

- `cargo test --bin specsync -- lifecycle_commit_scope_names_exactly_what_the_change_owns` passed.
- `cargo test` passed.

## Where these lessons go

- `specs/change/context.md`
