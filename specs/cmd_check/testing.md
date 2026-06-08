---
spec: cmd_check.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/check.rs` | cargo test commands::check | No inline tests found; add focused coverage for `cmd_check`, `IgnoreRules::load`, `build_comment_body`, `resolve_repo` before risky changes |
| `tests/integration.rs` | cargo test --test integration check_valid_project_passes | End-to-end fixture: `check_valid_project_passes` |
| `tests/integration.rs` | cargo test --test integration check_missing_source_file_fails | End-to-end fixture: `check_missing_source_file_fails` |
| `tests/integration.rs` | cargo test --test integration check_undocumented_export_warns | End-to-end fixture: `check_undocumented_export_warns` |
| `tests/integration.rs` | cargo test --test integration check_phantom_export_errors | End-to-end fixture: `check_phantom_export_errors` |
| `tests/integration.rs` | cargo test --test integration strict_turns_warnings_into_errors | End-to-end fixture: `strict_turns_warnings_into_errors` |
| `tests/integration.rs` | cargo test --test integration require_coverage_passes_when_met | End-to-end fixture: `require_coverage_passes_when_met` |
| `tests/integration.rs` | cargo test --test integration require_coverage_fails_when_below_threshold | End-to-end fixture: `require_coverage_fails_when_below_threshold` |
| `tests/integration.rs` | cargo test --test integration require_coverage_on_coverage_subcommand | End-to-end fixture: `require_coverage_on_coverage_subcommand` |
| `tests/integration.rs` (--fix) | cargo test --test integration fix_ | `fix_adds_undocumented_exports_to_spec`, `fix_does_not_duplicate_already_documented_exports`, `fix_creates_public_api_section_when_missing`, `fix_with_json_output`, `fix_does_not_duplicate_when_non_export_subsections_present`, `fix_near_miss_handles_levenshtein_typos`, `fix_dry_run_does_not_write_files`, `fix_backup_creates_backup_dir`, `fix_backup_preserves_original_on_success` |
| `tests/integration.rs` (suggestions/dry-run) | cargo test --test integration check_shows_fix_suggestions dry_run_without_fix_warns | `check_shows_fix_suggestions`, `dry_run_without_fix_warns`, `wildcard_reexport_with_fix_adds_all_symbols` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Incremental check with cache | 25 specs, 3 have changed since last check | `cmd_check` runs without `--force` | only 3 specs are validated; 22 are skipped via hash cache |
| Auto-fix undocumented exports | spec is missing export `pub fn new_function()` | `cmd_check` runs with `--fix` | the export is appended to the spec's Public API table with a generated description prompt and the file is rewritten |
| JSON output format | `--format json` is set | validation completes with errors and warnings | output is a single JSON object with `passed`, `errors`, `warnings`, `stale`, and `specs_checked` fields (verified by `fix_with_json_output`) |
| Backup before fix | a spec will be rewritten by `--fix` | `specsync check --fix --backup` is run | originals are copied to `.specsync/backup-fix/` before any write (`fix_backup_creates_backup_dir`) |
| Git staleness | a spec's source is N+ commits ahead of the spec's last commit, inside a git repo | `specsync check --stale N` is run | the spec is flagged "N commits behind source files" with per-file detail (uses `git_commits_since`) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| AI provider not available during `--fix` regen | Prints error per spec, continues with remaining specs | Keep or add a focused assertion before changing this behavior |
| Auto-fix changes a spec but validation still fails | Reports remaining errors, does not loop | Keep or add a focused assertion before changing this behavior |
| Hash cache file is corrupted | Falls back to full validation (cache miss) | Keep or add a focused assertion before changing this behavior |
| `--create-issues` with no GitHub repo | Prints error, skips issue creation | Keep or add a focused assertion before changing this behavior |
| `--stale` outside a git repo | No staleness output, no crash (the `is_git_repo` guard skips it) | Keep or add a focused assertion before changing this behavior |
| Validation has errors | Hash cache is NOT updated/saved (only saved when `total_errors == 0`) | Keep or add a focused assertion before changing this behavior |
| `--dry-run` without `--fix` | Prints a warning that dry-run has no effect, makes no changes | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- check --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/check.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
