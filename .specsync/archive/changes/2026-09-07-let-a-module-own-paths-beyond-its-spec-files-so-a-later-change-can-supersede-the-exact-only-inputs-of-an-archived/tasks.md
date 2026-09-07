---
change: let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived
artifact: tasks
---

# Tasks

- [x] `ModuleDefinition.owns`; parse `owns` in `parse_toml_modules_nested`, type it in the checked parser, write it in `config_to_toml`, keep a module carrying only `owns` round-tripping; update the three test literals in `validator.rs` and `generator.rs`.
- [x] `configured_module_owns_path` and `ownership_is_configurable`; `acceptance_input_owners` consults `owns` for declared modules ahead of `@exact:test` / `@exact:delivery`, never under `.specsync/`.
- [x] `validate_supersedes_semantics`: exact-only predecessor entry → eligible when the module owns the path now (`module_currently_owns_path`); module-signed entry → historical rule unchanged; refusal names the frozen label and the `owns` remedy.
- [x] Walk: `succession_claimants` + `successor_covers_input`; exact-only entries covered by any authenticated claimant; `exact_only_input_remediation_reason` names the supersede alternative where the path is configurable.
- [x] Tests: `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change`, `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module`, `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only`, `test_module_owns_alone_round_trips_and_is_not_a_files_mapping`; pinned exact-only message updated.
- [x] Deltas (`change`: REQ-change-020/024/036 + new REQ-change-095 + Error Cases; `config`: new REQ-config-013 + Config File Structure; `types`: Public API row), companions, site docs (`configuration.md`, `cli.md`).
