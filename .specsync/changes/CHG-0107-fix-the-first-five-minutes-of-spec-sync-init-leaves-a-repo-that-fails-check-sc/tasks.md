---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: tasks
---

# Tasks

## Bootstrap record (REQ-cmd-init-005, REQ-change-060, -061, -062)

- [x] Add `BOOTSTRAP_RECORD_PATH` and `BOOTSTRAP_RECORD_CANDIDATES` to `change`
- [x] Implement `record_bootstrap_paths` and export it
- [x] Implement `bootstrap_exempt_paths` with the four-condition guard
- [x] Implement `bootstrap_records` including the legacy `bootstrap_policy` shape
- [x] Implement `bootstrap_digest` / `policy_enforcement_projection` / `content_digest`
- [x] Implement `comparison_base_commit`, `commit_is_ancestor_of_head`,
      `project_path_is_absent_at`, `project_path_matches_digest`
- [x] Call `record_bootstrap_paths` from `execute_init`, collecting failure as a warning
- [x] Replace the `HEAD~1...HEAD` comparison-base fallback

## Stub exemption (REQ-change-063)

- [x] Add `STUB_SECTION_WARNING_PREFIX` with its coupling documented
- [x] Exempt stub warnings for sections no active change authored
- [x] Route the exemption through `IgnoreRules`

## Directory mappings (REQ-validator-010, REQ-cmd-issues-002)

- [x] Add `SourceSnapshot::Directory`
- [x] Return it from `snapshot_source_file` for a confined directory
- [x] Keep symlink and reparse-point rejection ahead of the directory branch
- [x] Add the `directory_mapping` branch to `validate_spec_content_internal`
- [x] Implement `directory_mapping_fix` with `DIRECTORY_MAPPING_FIX_LIMIT`
- [x] Implement `expand_directory_mapping` with `exclude_dirs` filtering, sort, dedupe
- [x] Make `generator::find_module_source_files` public

## Specs

- [x] Document `record_bootstrap_paths` in `specs/change`
- [x] Document `find_module_source_files` in `specs/generator`
- [x] Add the new requirements to `specs/cmd_init`, `specs/change`, `specs/validator`,
      `specs/cmd_issues`

## Verification

- [x] `cargo fmt --check` clean
- [x] `cargo clippy -- -D warnings` clean
- [x] Full `cargo test` green — 2203 unit, 331 integration, 0 failures
- [x] `specsync check --strict` green on this repository — 62 specs, 0 warnings, 0 failed

Sandbox drills run against a binary built from this branch after it lands; that pass
belongs to the RC loop, not to this change.
