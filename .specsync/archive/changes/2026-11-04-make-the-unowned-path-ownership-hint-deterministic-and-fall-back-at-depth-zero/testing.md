---
change: make-the-unowned-path-ownership-hint-deterministic-and-fall-back-at-depth-zero
artifact: testing
---

# Testing

Run with `cargo test --bin specsync approve_ownership_refusal`.

- `approve_ownership_refusal_at_depth_zero_falls_back_to_the_generic_message` (CHANGE-10.a) fails on 02818ce and on main f65dbed, and passes with the fix.
- `approve_ownership_refusal_names_every_undeclared_spec_that_lists_the_path_in_order` (CHANGE-10.b) fails on 0d04443/f65dbed and passes with the fix.
- `approve_ownership_refusal_matches_whole_directory_components` (`src/foo` vs `src/foobar`) fails on 02818ce and passes on main and with the fix.
- `approve_ownership_refusal_scores_a_glob_entry_by_its_directory` is coverage: it passes on main too, as Leif's review says globs score correctly.
- `approve_ownership_refusal_explains_a_listing_spec_without_a_module` and `approve_ownership_refusal_explains_a_declared_module_listing_outside_its_canonical_spec` cover the two cases of the `_ =>` arm and fail on main f65dbed.
- `approve_rejects_a_declared_path_owned_by_an_undeclared_module` still passes, so the same paths are refused.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-120 | `approve_ownership_refusal_at_depth_zero_falls_back_to_the_generic_message`, `approve_ownership_refusal_names_every_undeclared_spec_that_lists_the_path_in_order`, `approve_ownership_refusal_matches_whole_directory_components`, `approve_ownership_refusal_scores_a_glob_entry_by_its_directory`, `approve_ownership_refusal_explains_a_listing_spec_without_a_module`, `approve_ownership_refusal_explains_a_declared_module_listing_outside_its_canonical_spec`, `approve_ownership_refusal_names_the_declared_spec_to_edit`, `approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path`, `approve_rejects_a_declared_path_owned_by_an_undeclared_module` |
