---
spec: validator.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/validator.rs` | cargo test validator:: | `test_is_cross_project_ref`, `test_parse_cross_project_ref`, `test_find_spec_files_empty_dir`, `test_find_spec_files_nonexistent`, `test_find_spec_files_with_specs`, `test_validate_spec_missing_frontmatter` |
| `src/commands/issues.rs` | focused `snapshot_validation` command tests | `validate_spec_content_with_sources` consumes retained spec/source snapshots after ambient path replacement and does not disclose or validate replacement content |
| `src/validator.rs` | cargo test validator::malformed_gradle_settings_make_coverage_inconclusive | Checked coverage returns malformed or unconfined Gradle errors; compatibility coverage carries an inconclusive zero-percent result |
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
| Unconfined Gradle source root | raw drive-qualified module identity, interpolated/encoded path, unsafe Gradle manifest, unsupported/dynamic `setProjectDir`, or symlink/reparse component | a CLI/MCP gate calls `compute_coverage_checked` | the caller fails inconclusively before source traversal, partial totals, or generation |
| Retained spec snapshot validation | valid pre-read spec bytes and a logical spec path replaced after the snapshot | `validate_spec_content` is called | validation uses the pre-read spec bytes and opens neither the replaced spec path nor adjacent companions; mapped sources retain normal path behavior |
| Retained spec/source validation | retained spec bytes plus `SourceSnapshot` observations, with ambient paths replaced | `validate_spec_content_with_sources` is called | validation uses only supplied spec/source snapshots and ambient-free export extraction |
| Coverage ownership after ambient root replacement | caller-selected spec path now resolves through an attacker tree while the original project capability remains retained | checked ownership collection runs | the original spec mapping is used and replacement frontmatter is never parsed |
| Zero-config manifest/source detection replacement | omitted source directories plus a recognized manifest replaced after root retention | checked coverage autodetection runs | retained discovery rejects the replacement or uses only identity-continuous retained bytes; attacker-selected source roots never become authority |
| Explicit source directories | valid explicit `source_dirs` plus an unrelated unsafe autodetection input | retained config loading runs | configured roots are parsed without unrelated manifest/source autodetection |
| Nested retained config/manifest parent replacement | `.specsync` or a nested workspace parent is detached and replaced during acquisition | retained read or enumeration completes | reachability verification fails rather than mixing generations |
| Early and post-discovery race checkpoints | a gate caller reaches either checked-coverage checkpoint | root/spec/manifest/source input is replaced | checked coverage exits inconclusive with no outside read or partial total and the caller propagates the error |

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
| Unsupported or unconfined Gradle discovery | Checked coverage returns an error before source probing/traversal; all CLI/MCP gates remain non-success and content-free | Cover raw drive identifiers, interpolation, encoded traversal, unsafe recognized manifests, dynamic/unsupported `setProjectDir`, Unix symlink, and hosted-Windows reparse fixtures with unchanged outside sentinels |
| Caller-selected coverage spec path is replaced through the ambient root | Ownership comes only from the retained project capability and preserves the original mapping | Keep `retained_coverage_spec_mapping_ignores_an_ambient_root_replacement` |
| Selected-spec/source inventory reaches 100,000 entries | Limit succeeds and limit-plus-one fails before unbounded accumulation | Use injected small-limit unit coverage plus end-to-end selected-spec and source fixtures |
| Hosted-Windows source/junction replacement | Native junction fixture and path rendering remain portable | Prove the junction target, compare normalized paths, and run both checkpoints on hosted Windows |

The exact-head implementation adds
`retained_config_uses_configured_source_dirs_after_root_replacement`,
`retained_config_rejects_a_detached_parent_before_read`,
`retained_selected_specs_enforce_the_shared_entry_budget`,
`retained_spec_enumeration_is_bounded_before_returning_paths`,
`retained_coverage_file_read_rejects_preopen_regular_replacement`,
`retained_omitted_source_dirs_scan_after_a_malformed_manifest`, and
`retained_coverage_sources_reject_regular_directory_replacement_after_selection`. The focused
validator run passed 43 tests, and the full amended suite passed 1,948 unit plus 310 integration
tests. The Windows GNU cross-target compiled, but hosted-Windows runtime remains pending.

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/validator.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Preserve core-validation parity between `validate_spec` and `validate_spec_content`; path-based
  validation additionally reads adjacent companion markers. Keep the stronger no-ambient-source
  guarantee attached specifically to `validate_spec_content_with_sources`.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
- Do not treat historical Gradle, Windows cross-target, independent-review, trust, or CI results as
  evidence for the CHG-0063 amendment; rerun every required gate on the final exact tree.
