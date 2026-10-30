# Lesson bundle — fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fix custom Cargo library discovery and scoped npm generation for SpecSync 6.1.1
- **Kind**: Feature
- **Specs**: manifest, generator, cli, github, validator
- **Paths**: src/manifest/max_gradle_manifest_bytes.rs, src/manifest/tests.rs, src/generator.rs, tests/integration/commands/specsync_process.rs, Cargo.toml, Cargo.lock, action.yml, .github/workflows/ci.yml, .github/workflows/trust.yml, CHANGELOG.md, README.md, MIGRATION.md, SECURITY.md, docs/ADOPTING.md, site/src/content/docs, site/src/content/examples, src/validator/, specs/validator/
- **Acceptance**: Custom Cargo library target paths bind to the package module in default, JSON and named generation; scoped npm package names generate a truthful spec and cannot silently return success with no output; regression tests preserve existing owners and confinement; prepare version 6.1.1 and qualify the same immutable candidate on Ubuntu and macOS before Cargo or Homebrew publication.
- **Acceptance**: Declared project-root Cargo lib.rs/main.rs targets are bounded exact-file scan selections, not recursive scans of the project root. Literal resolved workspace members preserve whole-member boundaries and exact cross-member target ownership. Checked validator reads preserve no-follow file identity, confinement, filters and existing input budgets. Explicit directory selections retain their stated scope; workspace glob expansion is not added. Configuration and release documentation require a 6.1.1 reader for exact-file selections because 6.1.0 skips them even when its reduced coverage reports 100%.

## Evidence

- Verification commit: `dbf05b6c13604e36d8da06acae63193dd3aec7b3`
- Base commit: `8754ae17048742cc4150d1d98122b802ddc32e88`
- Verified by: `specsync check --spec cli --spec cmd_change --spec generator --spec github --spec manifest --spec validator`

## From the change's context.md

# Context

Main 8754ae1 includes the initial Cargo binding fix (#814). Late review still reproduces [lib].path="src/custom.rs" binding to custom rather than demo; named generation fails. A Node package named @scope/demo exits 0 without a spec because its full name is repeated as a filename beneath its module directory. Published 6.1.0 is immutable.

This repairs existing discovery and scaffold requirements. Preserve exact-target precedence, explicit mappings, existing ownership and retained-root confinement.

## From the change's testing.md

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

## Rustls patch qualification boundary

The fresh hosted audit of 0d8e321 found RUSTSEC-2026-0346/0347/0348/0349 in locked Rustls 0.23.45. Updating only that cached package to official 0.23.46 changes its version/checksum and leaves all other 255 package records and every dependency edge unchanged. The current RustSec snapshot edb58f5974b83eb8302332821cb6d389777aa351 contains 1,329 advisories; the repository audit task passes with zero vulnerabilities and no new ignored advisories. The before-lock diagnostic reproduces all four findings.

Independent review verified both crate archives against lock/index checksums and inspected actual upstream sources, resolved ring/std/logging/tls12 features and the unchanged minimum Rust version. No Brotli, ECH, custom-session-store exposure or PQ TLS behavior is claimed. Prior frozen tests do not qualify this changed dependency.

The first fresh Mac unit run reported 2,551 passes and one failure: the Git-monitor fixture reached an existing 120-second Git ls-files deadline before its expected fsmonitor diagnostic. The exact unchanged case passed on isolated retry, and both the previous qualified binary and patched binary passed the same exact case in a separate recorded comparison. No test was skipped and no deadline or security policy changed. Fresh integration, optimized binary and final hosted platform qualification remain separate gates.

The complete patched unit retry passed all 2,552 tests with eight test threads, zero failures and no skipped tests (receipt 001948-CODEX-996894). The complete patched integration suite passed all 459 tests (receipt 001344-CODEX-588e87). These are fresh Rust 1.89.0 runs with locked Rustls 0.23.46; the first native-Git timeout and old/new isolated comparisons remain recorded above.

Fresh patched qualification also passes formatting, Clippy with warnings denied, types, optimized release build, latest-released 6.1.0 forced strict verification and candidate 6.1.1 forced strict verification (62 specs, 121 source files, 100% file/LOC coverage), 52 release-validator tests and 29 required-CI tests. Runtime/lock hashes were checked around every step (receipt 002730-CODEX-987561); the optimized binary SHA-256 is a3c2da34d7dc3cfaa515aed73c72d5b926c3061a0cb68f9749f06f5856ba63cb. Hosted CI, scoped human review, archival, merge and immutable Ubuntu/macOS qualification remain pending.

## Where these lessons go

- `specs/manifest/manifest.3md`
- `specs/generator/generator.3md`
- `specs/cli/cli.3md`
- `specs/github/github.3md`
- `specs/validator/validator.3md`
