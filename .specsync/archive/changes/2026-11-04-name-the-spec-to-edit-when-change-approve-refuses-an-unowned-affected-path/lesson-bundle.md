# Lesson bundle: name-the-spec-to-edit-when-change-approve-refuses-an-unowned-affected-path

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Name the spec to edit when change approve refuses an unowned affected path
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change_tests.rs, src/change_tests/approve_ownership_names_the_spec.rs, specs/change, hi/change.md, INTENT.md
- **Acceptance**: When change approve refuses an affected path that no declared module owns, the error says which spec to edit for each such path: the undeclared spec that already lists it, or the declared spec to add it to.

## Evidence

- Verification commit: `0d0444346f28de069ccd307c69113de4044860c3`
- Base commit: `376d17cd44940bd1e241f1d15b64bb8542fe2e0a`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Leif asked for this in Grok Bot chat t811u on 2026-11-03, from Raven: "`change approve` refuses affected paths that no module owns (for example a new module's own `.spec.ts` files), with a fix that requires editing the module's files list; say which spec to edit in the error." The refusal listed the paths and said to "add the path to an owning spec's `files:` list" without naming the spec. In a repository with many specs the agent had to search for the right one.

The ownership rule from REQ-change-053 does not change. This change only words the refusal.

## From the change's testing.md

# Testing

`approve_ownership_refusal_names_the_declared_spec_to_edit` and `approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path` fail on main f82eb09, whose refusal names no spec, and pass with the fix. The existing `approve_rejects_a_declared_path_owned_by_an_undeclared_module` still passes, so the same paths are refused. Run with `cargo test --bin specsync approve_ownership_refusal`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-120 | `approve_ownership_refusal_names_the_declared_spec_to_edit`, `approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path`, `approve_rejects_a_declared_path_owned_by_an_undeclared_module` |

## Where these lessons go

- `specs/change/change.3md`
