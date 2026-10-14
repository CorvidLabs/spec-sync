---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: testing
---

# Testing

Focused unit and process regressions cover custom Cargo library files beneath/outside src, workspace libraries, scoped npm text/JSON/batch generation, valid YAML identity, companions, no-overwrite reruns and 100% coverage. Prove regressions fail before the fix. Run fledge lanes run verify and fledge trust verify, release-version/runtime validators and the docs build. Hosted release.yml supplies exact-SHA Ubuntu/macOS qualification; local macOS tests alone do not prove it.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-manifest-021` | `cargo_library_target_paths_remain_exact_and_confined` and `generate_custom_cargo_library_paths_keep_package_ownership` cover custom roots and package identity. The flat `generate_workspace_shared_library_paths_keep_package_ownership` is a causal before/after CLI regression. The original nested parser layout already worked and is a retention check, not a failing-before proof. Its extension covers targets outside the enclosing workspace; `generate_workspace_nested_escaping_library_paths_keep_package_ownership` actually fails against the hash-verified original committed PR816 subject. `generate_cargo_workspace_cross_member_targets_preserve_package_boundaries` actually fails against the preserved first corrected candidate before the sibling-boundary repair. Current qualification receipts are in the coordinator report; no passing status is inferred from test names. |
| `REQ-validator-044` | `exact_cargo_root_sources_keep_extension_and_explicit_directory_policy`, `exact_cargo_root_sources_use_retained_identity_and_input_budgets` and `exact_cargo_root_sources_reject_links_directories_and_special_files`; real CLI `generate_cargo_root_targets_measure_only_declared_files` confirms the target is measured without unrelated root files. |
| `REQ-generator-007` | `scoped_module_generation_entry_points_preserve_identity_and_leaf_paths` verifies ambient paths and built-in/custom companion references. `generate_scoped_npm_names_create_valid_specs_and_companions` verifies real CLI output, valid YAML, 100% coverage and no overwrite; it failed before the fix. `generate_write_failure_is_inconclusive_in_json_mode` verifies failed output cannot silently pass. |
| `REQ-cli-002` | Real process fixtures cover default, JSON and named generation. Existing provider-flag and agent command regressions remain in the full suite. |
| `REQ-github-002` | `python3 -S .github/scripts/validate-release-version.py` and `validate-workflow-runtime-pins.py` pass on the prepared 6.1.1 pins. Exact immutable Ubuntu/macOS qualification and promotion remain release gates rather than claimed local evidence. |

## Independent workspace/root-target review correction

Narrow workspace aggregation preserves ordinary per-package files and stable init.
Root-level Cargo targets use exact retained file selections instead of a project-wide
scan. Before/after CLI ownership and unrelated-output regressions, plus retained
file identity/resource-bound tests, qualify the correction. Fresh full verification
and independent review remain required; no human acceptance is inferred.

The first 15-file corrected freeze completed native stages in separate bounded
receipts after the combined lane timed out. Later cross-member source changes
need fresh qualification. Do not describe that earlier timeout as a full-lane
pass or its focused pre-edit receipts as proof of later frozen product bytes.

## v3 preparation boundaries and evidence

The new root-package `generate_cargo_root_package_cross_member_targets_preserve_package_boundaries`
regression targets the independent n1b case: root `[lib]` and `[[bin]]` files
inside two literal member src directories. The preserved v2 binary merges ordinary
same-named files across packages and named batches leave coverage incomplete.
Both member orders, init states and text/JSON/batch modes must pass with exact
owners after the correction. The earlier member-declared cross-target matrix
fails on main/v1, while the committed PR816 subject passes it; its historical
receipt is not a failing-before proof against that committed subject.

CHANGELOG/configuration docs state literal manifest-resolved member support and
the 6.1.1 minimum reader for exact-file source_dirs. Released 6.1.0 skips those
file entries; this is a compatibility boundary, not silent backward compatibility.
Glob member expansion and JSON generated-list order remain existing limitations;
JSON result contents are compared without depending on list order. Approved
definition artifacts and lifecycle evidence remain untouched pending the actual
human N1/N2 decision. Prior v2 native passes do not qualify changed v3 bytes.

## October 14 acceptance and qualification update

Leif explicitly answered: "Accept the tested fix; proceed through the release gates" to the prepared root-target/workspace/validator definition and implementation acceptance question. The accepted 17-file patch matches the independently reviewed freeze exactly. Recorded native evidence is 2,552 unit tests, 459 integration tests, 52 release-validator tests, 29 required-CI tests and 62 strict specs. A fresh pre-push gate passed on October 14. Runtime and test bytes remain unchanged while this supported definition amendment is materialized. Hosted CI and exact-SHA Ubuntu/macOS qualification remain pending and are not claimed from these local checks.
