# Testing

Run cargo test --test integration generate_cargo, cargo test generator:: and cargo test manifest::. Run full fledge lanes run verify before merge. Cover directory/flat same-name modules, arbitrary and nested target paths, matching package/target names, workspace prefixes, no overwrite and existing ownership. Confirm the original issue and both review reproductions with the optimized binary.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-manifest-020` | `src/manifest/tests.rs`: `cargo_binary_target_paths_remain_exact_and_merge_same_name_package_roots` passes for exact targets, same-name package roots and workspace prefixes. Seven `generate_cargo` integration tests and 24 optimized CLI cases pass with exact owners, no overwrite and 100% coverage. Full local verification passed 2,543 unit tests, 444 integration tests and both CI contract suites. |
