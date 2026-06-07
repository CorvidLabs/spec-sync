---
spec: validator.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/validator.rs` | cargo test validator:: | `test_is_cross_project_ref`, `test_parse_cross_project_ref`, `test_find_spec_files_empty_dir`, `test_find_spec_files_nonexistent`, `test_find_spec_files_with_specs`, `test_validate_spec_missing_frontmatter` |
| `tests/integration.rs` | cargo test --test integration check_valid_project_passes | End-to-end fixture: `check_valid_project_passes` |
| `tests/integration.rs` | cargo test --test integration check_missing_source_file_fails | End-to-end fixture: `check_missing_source_file_fails` |
| `tests/integration.rs` | cargo test --test integration check_undocumented_export_warns | End-to-end fixture: `check_undocumented_export_warns` |
| `tests/integration.rs` | cargo test --test integration check_phantom_export_errors | End-to-end fixture: `check_phantom_export_errors` |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | End-to-end fixture: `strict_turns_warnings_into_errors` |
| `tests/integration.rs` | cargo test --test integration invalid_frontmatter_reports_error | End-to-end fixture: `invalid_frontmatter_reports_error` |
| `tests/integration.rs` | cargo test --test integration missing_spec_dir_exits_cleanly | End-to-end fixture: `missing_spec_dir_exits_cleanly` |
| `tests/integration.rs` | cargo test --test integration missing_required_sections_reports_error | End-to-end fixture: `missing_required_sections_reports_error` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Valid spec passes | a spec with correct frontmatter, all required sections, and API table matching code exports | `validate_spec` is called | returns `ValidationResult` with empty errors and warnings |
| Spec documents non-existent export | a spec listing `` `nonExistent` `` in the Public API table | `validate_spec` is called | errors include "Spec documents 'nonExistent' but no matching export found in source" |
| Undocumented code export | source code exports `helperFn` but the spec does not list it | `validate_spec` is called | warnings include "Export 'helperFn' not in spec (undocumented)" |
| Cross-project dependency reference | a spec with `depends_on: ["corvid-labs/algochat@auth"]` | `validate_spec` is called locally | the cross-project ref is skipped (no error or warning) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec file unreadable | Error: "Cannot read spec" | Keep or add a focused assertion before changing this behavior |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" | Keep or add a focused assertion before changing this behavior |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) | Keep or add a focused assertion before changing this behavior |
| DB table not in schema | Error: "DB table not found in schema" | Keep or add a focused assertion before changing this behavior |
| Missing required section | Error: "Missing required section: ## SectionName" | Keep or add a focused assertion before changing this behavior |
| Dependency spec not found | Error: "Dependency spec not found" | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/validator.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
