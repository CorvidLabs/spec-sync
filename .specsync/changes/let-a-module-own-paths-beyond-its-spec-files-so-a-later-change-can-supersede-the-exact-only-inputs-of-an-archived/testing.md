---
change: let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived
artifact: testing
---

# Testing

- `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` — DISCRIMINATOR. A legacy (v1) bootstrap declares `tests/auth`, `Package.swift`, and `src/auth.rs`, is accepted and committed before any `owns` exists; its manifest signs `tests/auth/legacy.rs`, `tests/auth/deprecated.rs`, and the `tests/auth` directory entry (`non_file`) `@exact:test` and `Package.swift` `@exact:delivery`. Before the configuration grants the paths, `add_supersedes_obligation` refuses `auth` with the new message and persists nothing (on 404fe4d6 the refusal is "not a successor-eligible signed owner"). With `[modules."auth"] owns = ["tests/auth", "Package.swift"]`, a workflow-v2 successor adopts all five entries, edits the test, deletes the deprecated test, edits the package manifest, and goes approve → check → commit → check → review → finalize. Its manifest signs every owned path under `auth` (the directory entry and the `missing` deletion included), its tuples bind each frozen predecessor digest to `auth`, and the bootstrap is `SuccessorCovered` on `check_project` and `audit_project` before and after the archive commit. A later edit of the owned test is reported through the archived successor with its reason and does not offer the legacy reopen.
- `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module` — NEGATIVE. `owns = ["tests/auth"]` only: the test file is adopted, `Package.swift` is refused naming `@exact:delivery` and the `owns` remedy, the persisted record carries exactly the one obligation, and `src/auth.rs` (a module-signed entry) still resolves to `auth` by the historical rule.
- `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only` — unit CONTROL over `acceptance_input_owners`: a configured test file, the directory itself, and root delivery metadata resolve to the declared module; an undeclared module's paths, an unowned test tree, `fledge.toml`, and everything under `.specsync/` keep their exact class.
- `test_module_owns_alone_round_trips_and_is_not_a_files_mapping`, `test_config_to_toml_roundtrips_modules` — config round-trip with and beside `files`.
- `stale_accepted_change_error_names_exact_only_input_and_audited_reopen` — pins the amended exact-only message; `mapped_tests_remain_exact_only`, `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change`, and the `stale_accepted_change_error_names_*` family are unchanged CONTROLS.
- `fledge run lint` (clippy, `-D warnings`), `fledge lanes run pre-push`, `fledge lanes run verify`.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-095 | `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change`, `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module`, `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only` |
| REQ-change-020 | `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change`, `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module` |
| REQ-change-024 | `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` |
| REQ-change-036 | `stale_accepted_change_error_names_exact_only_input_and_audited_reopen`, `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` |
| REQ-config-013 | `test_module_owns_alone_round_trips_and_is_not_a_files_mapping`, `test_config_to_toml_roundtrips_modules` |
