---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: testing
---

# Testing

Focused unit and process regressions cover custom Cargo library files beneath/outside src, workspace libraries, scoped npm text/JSON/batch generation, valid YAML identity, companions, no-overwrite reruns and 100% coverage. Prove regressions fail before the fix. Run fledge lanes run verify and fledge trust verify, release-version/runtime validators and the docs build. Hosted release.yml supplies exact-SHA Ubuntu/macOS qualification; local macOS tests alone do not prove it.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-manifest-021` | `src/manifest/tests.rs`: `cargo_library_target_paths_remain_exact_and_confined` passes; `generate_custom_cargo_library_paths_keep_package_ownership` proves text/JSON/batch ownership across custom roots and workspace prefixes, and failed before the fix. |
| `REQ-generator-007` | `scoped_module_generation_entry_points_preserve_identity_and_leaf_paths` verifies ambient paths and built-in/custom companion references. `generate_scoped_npm_names_create_valid_specs_and_companions` verifies real CLI output, valid YAML, 100% coverage and no overwrite; it failed before the fix. `generate_write_failure_is_inconclusive_in_json_mode` verifies failed output cannot silently pass. |
| `REQ-cli-002` | Real process fixtures cover default, JSON and named generation. Existing provider-flag and agent command regressions remain in the full suite. |
| `REQ-github-002` | `python3 -S .github/scripts/validate-release-version.py` and `validate-workflow-runtime-pins.py` pass on the prepared 6.1.1 pins. Exact immutable Ubuntu/macOS qualification and promotion remain release gates rather than claimed local evidence. |
