---
change: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
artifact: research
---

# Research

The 36 lints, by kind, with lint names taken from clippy's JSON output on ab688246. Each fix keeps the test's meaning.

| Lint | Count | Where | Fix |
|---|---|---|---|
| `field_reassign_with_default` | 19 | `semantic_application_resolves_registry_backed_canonical_paths.rs` (12), `canonical_resolution_tolerates_inert_legacy_registry_stub.rs` (3), `approve_rejects_…_undeclared_module.rs`, `substantive_plan.rs`, `generator.rs`, `validator/tests.rs` (1 each) | Build the value with `Struct { field, ..Default::default() }` |
| `manual_contains` (`iter().any()`) | 5 | `approve_rejects_…_undeclared_module.rs` | `.contains(&expected)` |
| `type_complexity` | 2 | `sdd_version.rs:91` (a `#[cfg(test)]` thread-local hook), `fast_archived_reads.rs:1433` | A `#[cfg(test)]` type alias |
| `items_after_test_module` | 2 | `commands/scaffold.rs`, `commands/init.rs` | Move the test module after the last item. In `init.rs`, move `print_legacy_policy_notice` above it instead |
| `nonminimal_bool` | 2 | `merge_driver.rs` tests | Apply the simplified form |
| `needless_return` | 1 | `consider_accepted_evidence_anchor.rs:941` (`#[cfg(test)]` helper) | Drop `return` |
| `unused_mut` (rustc) | 1 | `substantive_plan.rs:3863` | Drop `mut` |
| `collapsible_if` | 1 | `tests/integration/change.rs:19` | Collapse with `&&` |
| `cloned_ref_to_slice_refs` | 1 | `hash_cache.rs:955` test | `std::slice::from_ref(&spec_path)` |
| `bool_comparison` | 1 | `commands/init.rs:871` test | `!…exists()` |
| `assertions_on_constants` | 1 | `agents.rs:1752` | `const { assert!(AGENT_ARTIFACT_TEMPLATE_VERSION >= 6) }`, which turns the runtime check into a compile-time one |

Clippy finished every target without stopping early: the bin-test target reported 35 and the integration target 1. The full list is in the measurement log, and the build used `nice -n 10`, `-j 4`, about 85 s.

The other open PRs (#918, #901, #963, #920, #943, #967, #900, #969, #964, #934, …) may add test code that `--all-targets` flags. Once this lands, each of them has to clear its own lints when it merges main in. That's the intended effect, and it is listed as a risk in plan.md.
