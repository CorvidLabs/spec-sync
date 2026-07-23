---
spec: validator.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/validator.rs` | cargo test validator:: | `test_is_cross_project_ref`, `test_parse_cross_project_ref`, `test_find_spec_files_empty_dir`, `test_find_spec_files_nonexistent`, `test_find_spec_files_with_specs`, `test_validate_spec_missing_frontmatter` |
| `src/commands/issues.rs` | focused `snapshot_validation` command tests | `validate_spec_content_with_sources` consumes retained spec/source snapshots after ambient path replacement and does not disclose or validate replacement content |
| `src/validator.rs` | cargo test validator::malformed_gradle_settings_make_coverage_inconclusive | Checked coverage returns the Gradle parse error; compatibility coverage carries an inconclusive zero-percent result |
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
| Malformed Gradle settings | a source tree with an unterminated Gradle `include` declaration | a CLI/MCP gate calls `compute_coverage_checked` | coverage is inconclusive and the caller fails instead of reporting partial totals |
| Retained spec snapshot validation | valid pre-read spec bytes and a logical spec path replaced after the snapshot | `validate_spec_content` is called | validation uses the pre-read spec bytes and opens neither the replaced spec path nor adjacent companions; mapped sources retain normal path behavior |
| Retained spec/source validation | retained spec bytes plus `SourceSnapshot` observations, with ambient paths replaced | `validate_spec_content_with_sources` is called | validation uses only supplied spec/source snapshots and ambient-free export extraction |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec file unreadable | Error: "Cannot read spec" | Keep or add a focused assertion before changing this behavior |
| `validate_spec_content` receives pre-read bytes | Validates those exact spec bytes; `spec_path` and adjacent companions are not opened, while mapped sources retain normal path behavior | Keep core spec-content tests |
| `validate_spec_content_with_sources` receives spec/source snapshots | Treats `SourceSnapshot` observations as authoritative; no spec/source reopen or ambient wildcard resolution | Keep both issue-command replacement regressions and the exports supplied-content regression |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" | Keep or add a focused assertion before changing this behavior |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) | Keep or add a focused assertion before changing this behavior |
| DB table not in schema | Error: "DB table not found in schema" | Keep or add a focused assertion before changing this behavior |
| Missing required section | Error: "Missing required section: ## SectionName" | Keep or add a focused assertion before changing this behavior |
| Dependency spec not found | Error: "Dependency spec not found" | Keep or add a focused assertion before changing this behavior |
| Static HTML mapped or unmapped | Coverage denominator remains one and a 100% gate distinguishes `1/1` from `0/1` | Keep CLI fixtures for both cases |
| Generated companion marker | Warning includes artifact path and source line; strict mode fails | Cover every supported artifact plus fenced and similar-prose negatives |
| Built-in design markers | Layout, Components, Tokens, and Assets placeholders each produce a distinct warning | Keep the generated template lines and validator marker table in parity |
| Malformed or unreadable Gradle settings | Checked coverage returns an error; compatibility coverage remains callable with a zero-percent inconclusive report | Keep gate callers on `compute_coverage_checked` and cover text/JSON or MCP failure output when it changes |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/validator.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Preserve core-validation parity between `validate_spec` and `validate_spec_content`; path-based
  validation additionally reads adjacent companion markers. Keep the stronger no-ambient-source
  guarantee attached specifically to `validate_spec_content_with_sources`.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
