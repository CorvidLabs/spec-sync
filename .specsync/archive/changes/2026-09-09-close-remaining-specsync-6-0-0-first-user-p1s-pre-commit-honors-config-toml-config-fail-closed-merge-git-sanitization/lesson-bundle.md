# Lesson bundle — close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Close remaining SpecSync 6.0.0 first-user P1s: pre-commit honors config, TOML config fail-closed, merge git sanitization, and 5.x upgrade docs
- **Kind**: BugFix
- **Specs**: hooks, config, merge, git_utils, change
- **Paths**: src/hooks.rs, src/config.rs, src/merge.rs, src/git_utils.rs, src/change.rs, src/change_tests.rs, tests/integration/config.rs, tests/integration/quickstart.rs, MIGRATION.md, docs/6-0-confidence-report.md, CHANGELOG.md
- **Acceptance**: init then add-spec then check then hooks install then git commit succeeds without --no-verify; malformed TOML (garbage, empty file, directory-as-config, invalid enforcement enum) makes rules/rehash/compact/deps/archive-tasks/view exit 1 with could not be loaded, same as malformed JSON; no-config-file still uses defaults; unmerged_paths spawns git through git_cmd so GITHUB_TOKEN/AWS_SECRET_ACCESS_KEY/SPECSYNC_TEST_SECRET are absent from the child; v1 Verifying next_action names verify then accept then archive; uncovered-path remediation uses --kind bug-fix; MIGRATION.md has a copy-pasteable v1 close-out with merge-then-archive, commit of workflow-v2-baseline.json and adoption-report.json, names both change check and that verify no longer runs verification_commands, and states adopt is silent on committed still-active v1; confidence-report rows for #653 and MCP sanitization match the tree

## Evidence

- Verification commit: `5c3bbbce6473994dc8ad50d0feacb34b2555b959`
- Base commit: `7df304a20edd77a5cc32396a1239b08577605077`
- Verified by: `specsync check --spec change --spec config --spec git_utils --spec hooks --spec merge`

## From the change's context.md

# Context

Independent proving after #770/#771 found three remaining first-user P1s on `main` (`7df304a`, which already includes #772 and #773). Do not revert the Action rc.14 pin. Do not touch #772/#773/#628.

## P1s closed here

1. First-user `init` → `add-spec` → `check` (0 errors, warnings) → `hooks install` → `git commit` fails because the generated pre-commit hook hardcodes `specsync check --strict`. Default 6.0 enforcement is strict on errors, not warnings. The hook must run `specsync check` so it honors `.specsync/config.toml`.
2. #653 is half-fixed: JSON parse-fail sets `load_error` and `load_config` refuses; TOML still goes through the silent line scanner, so `rules` / `rehash` / `compact` / `archive-tasks` / `deps` exit 0 over garbage. Choke point is `parse_config_content_checked` after reading TOML.
3. `merge::unmerged_paths` still spawns `Command::new("git")` with the full parent environment, so the MCP/check path via `cached_unmerged_paths` forwards `GITHUB_TOKEN`. It must use `git_cmd` (made `pub(crate)`).

Also in this package: workflow-v1 `Verifying` `next_action` names `verify` → `accept` → `archive` instead of v2 `check`/`review`/`finalize`; uncovered-path remediation uses `--kind bug-fix`; `MIGRATION.md` gets a copy-pasteable v1 close-out (F1/F4/F5/F6); confidence-report rows for #653 and MCP sanitization stop claiming a full fix.

## Constraints

- One change package, every touched `src/` path owned.
- Do not create tags, dispatch `promote`, `cargo publish`, touch Homebrew, force-push `main`, merge this PR, or close issues.
- Do not convert `change.rs` commit paths that rely on `GIT_AUTHOR_NAME`.
- Do not expand: fence-blindness, generate minting invalid specs, `fix_near_miss_required_headers`, nested-git half, interrupted-finalize attempts-ledger, CRLF.

## From the change's design.md

# Design

Local, fail-closed fixes. No new verbs.

- Generated pre-commit hook runs `specsync check` (and `specsync --root <project> check` in the managed block), not `check --strict`. Comment names `.specsync/config.toml` and the real default (strict on errors, warnings pass unless `--strict` or config says otherwise).
- `load_toml_config` parses through `parse_config_content_checked`. On `Err` or empty content it sets `load_error` with the same wording as JSON. `validate_toml_config_types` rejects an unknown `enforcement` enum instead of warning and keeping the default. Directory-as-config already fails via unreadable-file `load_error`.
- `git_cmd` is `pub(crate)`. `unmerged_paths` uses it and does not set `current_dir` again (`git_cmd` already does). Do not convert `change.rs` commit paths that need `GIT_AUTHOR_NAME`.
- `summarize_change` Verifying arm: if `workflow_version < 2`, next_action is `verify` then `accept` then `archive`. Leave the Accepted arm alone.
- Uncovered-path remediation uses `--kind bug-fix`.
- `MIGRATION.md` step 4 gets a literal v1 close-out: merge-then-archive, commit `.specsync/workflow-v2-baseline.json` and `.specsync/adoption-report.json` after adopt, names both `change check` and that `verify` no longer runs `verification_commands`, and states `adopt` is silent on a committed still-active v1.

## From the change's testing.md

# Testing

Fail-then-pass tests for each P1, plus the docs/next-action pins.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-hooks-002 | `install_precommit_creates_hook_file`, `install_precommit_uses_configured_hooks_path_and_preserves_exit_zero` (hook body has `specsync check` / `--root … check` without `--strict`) |
| REQ-hooks-003 | `init_add_spec_hooks_install_then_commit_succeeds_without_strict` |
| REQ-config-015 | `test_load_config_malformed_toml_sets_load_error`, `test_load_config_empty_toml_sets_load_error`, `test_load_config_directory_as_toml_sets_load_error`, `test_load_config_invalid_enforcement_enum_sets_load_error`, `malformed_toml_config_refuses_rules_rehash_compact_and_view`, `malformed_json_config_refuses_rules_rehash_compact_and_view`, `absent_config_still_runs_with_defaults` |
| REQ-git-utils-007 | `git_cmd_does_not_forward_sentinel_secrets` |
| REQ-merge-003 | `unmerged_paths_uses_sanitized_git_cmd`, `unmerged_paths_is_unknown_outside_a_git_repository` |
| REQ-change-100 | `workflow_v1_verifying_next_action_names_verify_accept_archive` |
| REQ-change-101 | `uncovered_paths_error_names_the_escape_hatch_and_ignore_precedence` (`--kind bug-fix`) |

## Where these lessons go

- `specs/hooks/context.md`
- `specs/config/context.md`
- `specs/merge/context.md`
- `specs/git_utils/context.md`
- `specs/change/context.md`
