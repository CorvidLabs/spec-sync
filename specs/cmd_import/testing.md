---
spec: cmd_import.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/import.rs` | cargo test commands::import | No inline tests found; add focused coverage for `cmd_import`, `load_config`, `generate_companion_files`, `resolve_repo` before risky changes |
| `tests/integration.rs` | cargo test --test integration import_without_args_or_flags_shows_error | End-to-end fixture: `import_without_args_or_flags_shows_error` |
| `tests/integration.rs` | cargo test --test integration import_from_dir_imports_markdown_files | End-to-end fixture: `import_from_dir_imports_markdown_files` |
| `tests/integration.rs` | cargo test --test integration import_from_dir_skips_existing_specs | End-to-end fixture: `import_from_dir_skips_existing_specs` |
| `tests/integration.rs` | cargo test --test integration import_from_dir_nonexistent_directory_errors | End-to-end fixture: `import_from_dir_nonexistent_directory_errors` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Import GitHub issue | `specsync import github 42` | `cmd_import` runs | fetches issue #42, creates spec from its title and body |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Invalid source type | Exits 1 with supported list | Keep or add a focused assertion before changing this behavior |
| Spec already exists | Exits 1 | Keep or add a focused assertion before changing this behavior |
| Fetch fails | Exits 1 with error | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- import --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/import.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
