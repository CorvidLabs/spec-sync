---
change: let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived
artifact: plan
---

# Plan

1. `types` / `config`: add `owns` to `ModuleDefinition`; parse, validate, and serialize it; round-trip tests.
2. `change`: `configured_module_owns_path` / `ownership_is_configurable`; consult `owns` in `acceptance_input_owners` ahead of the reserved exact classes for declared modules only.
3. `change`: in `validate_supersedes_semantics`, admit a module for an exact-only predecessor entry when it owns the path now (`module_currently_owns_path`); keep the historical rule for entries a module signed; name the `owns` remedy.
4. `change`: in the walk, read the claimants of a changed exact-only entry from the successors' declared obligations (`succession_claimants`) and judge each with `successor_covers_input`, extracted from the per-owner loop so both paths share every check; amend the exact-only diagnostic.
5. Regression tests in `src/change_tests.rs`: the bootstrap scenario (edit, delete, package manifest; finalize; covered on both surfaces before and after the archive commit; directory entry owned), the negative (a path the configuration does not grant refuses without persisting), the unit control over `acceptance_input_owners`.
6. Deltas for `change`, `config`, `types`; companions; site docs. `fledge run lint`, `fledge lanes run pre-push`, `fledge lanes run verify`. Commit; open the PR against the #753 branch citing the swift-algorand reproduction, #753, #751, #688; stop before `approve`.
