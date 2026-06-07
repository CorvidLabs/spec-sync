---
spec: deps.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/deps.rs` | cargo test deps:: | `test_extract_module_from_dep_path`, `test_build_dep_graph_empty`, `test_build_dep_graph_basic`, `test_validate_no_errors`, `test_validate_missing_dep`, `test_detect_circular_deps` |

## Coverage Gaps

- Integration gap: add a fixture for "Detect missing dependency" before changing user-visible CLI output, generated files, or error handling in deps.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Detect missing dependency | spec A declares `depends_on: [specs/nonexistent/nonexistent.spec.md]` | `validate_deps` is called | report contains an error about the missing dependency spec |
| Detect circular dependency | spec A depends on B and spec B depends on A | `validate_deps` is called | report's `cycles` field contains the chain `[A, B, A]` |
| Extract Rust imports | a file containing `use crate::config::load_config;` | `extract_imports(path, content)` is called | returns a set containing `"config"` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Source file unreadable | Skipped during import extraction | Keep or add a focused assertion before changing this behavior |
| Spec frontmatter unparseable | Module excluded from dependency graph | Keep or add a focused assertion before changing this behavior |
| No specs found in specs_dir | Returns empty graph and clean DepsReport | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/deps.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
