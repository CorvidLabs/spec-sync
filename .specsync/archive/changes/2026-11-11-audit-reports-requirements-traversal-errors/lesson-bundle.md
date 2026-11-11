# Lesson bundle: audit-reports-requirements-traversal-errors

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Audit reports requirements traversal errors
- **Kind**: BugFix
- **Specs**: validator
- **Paths**: src/validator/requirement_ids.rs, tests/
- **Acceptance**: an unreadable subdirectory under specs/ produces an audit error naming the path instead of being skipped

## Evidence

- Verification commit: `6ab73f9421bde01e2c415727e7f34105b894f0e3`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec validator`

## From the change's context.md

# Context

Issue #873 (follow-up from #865's review): `duplicate_requirement_id_errors`
walks the specs tree with `WalkDir(...).filter_map(Result::ok)`, so a
directory it cannot traverse is dropped silently and `change audit` passes
without inspecting the requirements.md files below it. Unreadable *files* are
already reported; unreadable *directories* were the gap. The fix turns each
traversal error into an audit error naming the path.

## From the change's testing.md

# Testing

Unit test: an unreadable specs subdirectory produces a traversal error naming
the path.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-validator-046 | change_tests unit test with a chmod-0 subdirectory |

## Where these lessons go

- `specs/validator/validator.3md`
