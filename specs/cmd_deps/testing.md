---
spec: cmd_deps.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/deps.rs` | cargo test commands::deps | Command wrapper has no inline tests (format dispatch + private `render_mermaid`/`render_dot`); cover end-to-end before risky changes |
| `src/deps.rs` graph + validation | cargo test deps::tests | `test_build_dep_graph_empty`, `test_build_dep_graph_basic`, `test_validate_no_errors`, `test_validate_missing_dep` |
| `src/deps.rs` cycles + ordering | cargo test deps::tests | `test_detect_circular_deps`, `test_detect_three_node_cycle`, `test_topological_sort_valid`, `test_topological_sort_cycle` |
| `src/deps.rs` imports | cargo test deps::tests | `test_undeclared_rust_import`, `test_undeclared_ts_import`, `test_undeclared_python_import`, `test_self_import_not_flagged`, `test_declared_import_not_flagged` |
| `tests/integration.rs` | cargo test --test integration dependency_spec_not_found_errors | End-to-end: a spec's `depends_on` references a nonexistent spec → error |
| `tests/integration.rs` strict gate | cargo test --test integration deps_strict_gates_on_undeclared_imports | End-to-end: undeclared import under `--strict` exits 1; the stderr note is present for non-JSON and absent for `--format json` |

## Coverage Gaps

- The private `render_mermaid`/`render_dot` diagram output and the wrapper's exit-on-errors path have no direct test. Add a CLI-level assertion before changing diagram syntax or the exit behavior.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Mermaid output | clean dep graph, `--mermaid` | `cmd_deps(root, false, Text, true, false)` | prints `graph LR` with sorted nodes/edges; missing targets shown as dashed `❌` nodes |
| DOT output | clean dep graph, `--dot` | `cmd_deps(root, false, Text, false, true)` | prints `digraph specs { … }` with sorted nodes/edges |
| Valid graph, text format | no errors/warnings, no cycles | `cmd_deps(root, false, Text, false, false)` | prints "All dependency declarations are valid." plus a "Build order: …" line |
| Cycle detected | A depends on B, B depends on A | `cmd_deps(root, false, Text, false, false)` | prints cycle error and exits 1 |
| Undeclared import under `--strict` | a module imports a spec not in its `depends_on` | `cmd_deps(root, true, Text, false, false)` | warning printed; `--strict mode: N dependency warning(s) treated as errors` note on stderr; exits 1 |
| Undeclared import under `--strict --format json` | same undeclared-import warning | `cmd_deps(root, true, Json, false, false)` | warning is in the JSON `warnings` array; **no** stderr note (JSON stays machine-readable); exits 1 |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Circular dependency | Error printed, exits 1 | Keep or add a focused assertion before changing this behavior |
| Missing dependency spec | Error printed (`missing_deps`), exits 1 (covered by `dependency_spec_not_found_errors`) | Keep or add a focused assertion before changing this behavior |
| Empty dep graph in diagram mode | Prints `depends_on` hint to stderr (diagram mode only) | Keep or add a focused assertion before changing this behavior |
| Undeclared import (no `--strict`) | Reported as a warning; does NOT force exit 1 (only `report.errors` do) | Keep or add a focused assertion before changing this behavior |
| Undeclared import under `--strict` | Warning forces exit 1; stderr note present for non-JSON, suppressed for JSON | `deps_strict_gates_on_undeclared_imports` (asserts exit 1 + note present non-JSON / absent JSON) |

## Reviewer Checklist

- Run `cargo run -- deps --help` and confirm the help text still names the documented flags (`--format`, `--strict`, `--mermaid`, `--dot`).
- Run `cargo test deps` when changing the delegate; run `cargo test commands::deps` when changing the wrapper.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message or diagram syntax changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
