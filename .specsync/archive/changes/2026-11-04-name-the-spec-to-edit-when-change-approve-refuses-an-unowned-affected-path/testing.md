---
change: name-the-spec-to-edit-when-change-approve-refuses-an-unowned-affected-path
artifact: testing
---

# Testing

`approve_ownership_refusal_names_the_declared_spec_to_edit` and `approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path` fail on main f82eb09, whose refusal names no spec, and pass with the fix. The existing `approve_rejects_a_declared_path_owned_by_an_undeclared_module` still passes, so the same paths are refused. Run with `cargo test --bin specsync approve_ownership_refusal`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-120 | `approve_ownership_refusal_names_the_declared_spec_to_edit`, `approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path`, `approve_rejects_a_declared_path_owned_by_an_undeclared_module` |
