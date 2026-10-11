# Testing

Run cargo test --test integration generate_cargo, cargo test generator:: and cargo test manifest::. Run full fledge lanes run verify before merge. Cover directory/flat same-name modules, arbitrary and nested target paths, matching package/target names, both pathless binary defaults, workspace prefixes, no overwrite and existing ownership. Confirm the original issue and all review reproductions with the optimized binary.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-manifest-020` | `src/manifest/tests.rs`: `cargo_binary_target_paths_remain_exact_and_merge_same_name_package_roots` and `cargo_implicit_binary_paths_include_flat_and_nested_defaults` pass for exact targets, same-name package roots, both defaults and workspace prefixes. Eleven Cargo process regressions, 51 generator tests and 61 manifest tests pass, including differently named existing owners. Full local verification and optimized reproductions must pass again before completion. |
