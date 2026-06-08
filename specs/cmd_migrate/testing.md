---
spec: cmd_migrate.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/migrate.rs` | cargo test commands::migrate | `apply_create_directories_creates_v4_layout`, `apply_create_directories_is_idempotent` (step-apply unit tests). Other `apply_*` steps are exercised only via the integration flow below |
| `tests/integration.rs` | cargo test --test integration migrate_full_v3_to_v4 | End-to-end fixture: `migrate_full_v3_to_v4` |
| `tests/integration.rs` | cargo test --test integration migrate_check_passes_after_migration | End-to-end fixture: `migrate_check_passes_after_migration` |
| `tests/integration.rs` | cargo test --test integration migrate_idempotent_rerun_is_noop | End-to-end fixture: `migrate_idempotent_rerun_is_noop` |
| `tests/integration.rs` | cargo test --test integration migrate_dry_run_no_side_effects | End-to-end fixture: `migrate_dry_run_no_side_effects` |
| `tests/integration.rs` | cargo test --test integration migrate_json_output_format | End-to-end fixture: `migrate_json_output_format` |
| `tests/integration.rs` | cargo test --test integration migrate_no_project_fails | End-to-end fixture: `migrate_no_project_fails` |
| `tests/integration.rs` | cargo test --test integration migrate_no_backup_flag | End-to-end fixture: `migrate_no_backup_flag` |
| `tests/integration.rs` | cargo test --test integration migrate_partial_recovery | End-to-end fixture: `migrate_partial_recovery` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Fresh migration from 3.x | a project with `specsync.json` in root, specs with `lifecycle_log` in frontmatter | `specsync migrate` runs | config converts to `.specsync/config.toml`, lifecycle logs extracted to `.specsync/lifecycle/`, frontmatter cleaned, backup created in `.specsync/backup-3x/` |
| Already migrated project | a project with `.specsync/version` containing `4.0.0` | `specsync migrate` runs | outputs "Already at v4.0.0 — nothing to migrate" and exits 0 |
| Dry run | a 3.x project | `specsync migrate --dry-run` runs | outputs an ordered migration plan showing what would change (files moved, frontmatter fields removed, directories created) without modifying any files |
| Partial migration recovery | a project where a previous migrate crashed after step 4 (config relocated but registry not yet moved) | `specsync migrate` runs again | steps 1-4 report "already done", steps 5+ execute normally |
| Spec with no lifecycle_log | a spec that has never had a lifecycle transition | migrate runs the extract step | no `.specsync/lifecycle/{module}.json` is created for that spec (only specs with history get files) |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No `specsync.json` found and no `.specsync/config.json` | Error: "No spec-sync project found. Run `specsync init` first" | Keep or add a focused assertion before changing this behavior |
| Permission denied writing to `.specsync/` | Error with path and suggestion to check permissions | Keep or add a focused assertion before changing this behavior |
| Spec file with malformed frontmatter | Warning: skip that spec's lifecycle extraction, continue with others, report in summary | Keep or add a focused assertion before changing this behavior |
| Disk full during backup | Error: "Backup failed — original files untouched. Free disk space and retry" | Keep or add a focused assertion before changing this behavior |
| `--no-backup` with destructive steps | Proceed without backup (user opted out) | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- migrate --help` and confirm the help text still names the documented flags and behavior.
- Run the narrow source command above before the full suite when changing `src/commands/migrate.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
