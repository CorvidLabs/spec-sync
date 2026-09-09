# Lesson bundle — close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Close the SpecSync 6.0.0 P1 release defects found in overnight proving
- **Kind**: BugFix
- **Specs**: change, cli_args, cmd_check, cmd_generate, cmd_new, cmd_scaffold, config, generator, git_utils, cli, parser, mcp
- **Paths**: src/change.rs, src/change_tests.rs, src/lifecycle-validation-limits.json, src/cli.rs, src/commands/check.rs, src/commands/generate.rs, src/commands/new.rs, src/commands/scaffold.rs, src/config.rs, src/generator.rs, src/git_utils.rs, src/main.rs, src/mcp.rs, src/parser.rs, tests/integration/check.rs, tests/integration/commands.rs, tests/integration/config.rs, tests/integration/fix.rs, tests/integration/quickstart.rs, site/src/content/docs/cli.md, site/src/content/docs/mcp-security.md, site/src/content/docs/quickstart.md, README.md, CHANGELOG.md
- **Acceptance**: cargo publish --dry-run compiles because lifecycle limits are bundled under src/; specsync new emits all seven init-required sections and refuses reserved names; malformed JSON config exits 1 on rules/rehash/compact/deps/archive-tasks/view; git children drop GITHUB_TOKEN and GIT_DIR and pin GIT_CEILING_DIRECTORIES; deleting or emptying verification-attempts.json refuses adopted finalization; CRLF definition payloads hash as LF; fenced Public API examples are not documented exports and check --fix ignores fenced ### headings; check --spec NAME parses; generate exits 1 when unspecced modules have no files; scaffold --dir cannot escape the project root; README and site quick start succeed without --strict on a stub scaffold.

## Evidence

- Verification commit: `dbcd64156437c32ca22e0fbc2bb973c6f0283fdf`
- Base commit: `0d0251bf62b30696a710a2a421becac02405539b`
- Verified by: `specsync check --spec change --spec cli --spec cli_args --spec cmd_check --spec cmd_generate --spec cmd_new --spec cmd_scaffold --spec config --spec generator --spec git_utils --spec mcp --spec parser`

## From the change's context.md

# Context

Overnight proving against `0d0251bf` produced 196 findings, 8 of them P1. Fixes were drafted in isolated worktrees that never reached GitHub; this package reconstructs them onto one branch and takes them through the 6.0 lifecycle.

## P1s closed here

1. `cargo publish` could not compile: `include_str!` of `.github/scripts/lifecycle-validation-limits.json` is outside Cargo.toml's `/src/**` include set.
2. `specsync new` emitted 4 of the 7 sections `init` configures.
3. Malformed JSON config silently fell back to defaults in `rules`/`rehash`/`compact`/`deps`/`archive-tasks`/`view` (#653, incomplete #583).
4. MCP/git children inherited `GITHUB_TOKEN` and `GIT_DIR`.
5. `ship`/`finalize` refused a CRLF checkout (post-move archive digest vs LF blob).
6. README / site quick start failed at the first `check --strict` after `add-spec` with no source.
7. Deleting `verification-attempts.json` bypassed the empty-ledger adoption guard (#656).
8. Public API readers counted fenced example backticks as documented exports; `--fix` used a fenced `###` as an insert target (#768.3).

Also closed on the same tree: `check --spec` as a real flag (evidence already recorded that form), `generate` silent no-op on empty file lists, `scaffold --dir` path escape.

## Constraints

- One change package, every touched `src/` path owned.
- Do not create tags, dispatch `promote`, `cargo publish`, touch Homebrew, force-push `main`, or merge the PR.
- `blank_fenced_code` must preserve byte length: `--fix` uses blanked offsets as indexes into the original.

## Out of scope

#605 coverage gate skip, #675/#690 messages, #439 performance, #434 unknown-field preservation, #532 multi-clone approvals, #768.1/#768.2 remaining `--fix` table-scanner cases, `report --require-coverage` stale skip.

## From the change's design.md

# Design

Local, fail-closed fixes. No new verbs.

- Bundle `lifecycle-validation-limits.json` under `src/` so crates.io's include set can compile `include_str!`. Keep the CI copy byte-identical; a unit test pins that.
- `new` calls `generator::generate_spec` and `validate_scaffold_module_name` instead of a four-section private skeleton and `chrono_lite_today`.
- JSON parse failure in `load_json_config` sets `load_error` with the same wording as an unreadable file. `load_config` already refuses via `refuse_unloadable_config`.
- One `git_cmd(root)` helper: `env_clear`, PATH/locale/home/temp allowlist, `GIT_CEILING_DIRECTORIES` = parent of canonical root. Every production git spawn in `git_utils` uses it.
- `canonical_definition_artifact_payload` folds `\r\n` → `\n` (lone `\r` stays) before the tasks.md checkbox rewrite, matching `canonical_delta_body`.
- Missing `verification-attempts.json` is treated as an empty ledger at accept; a Verifying change refuses to recreate a missing ledger.
- `blank_fenced_code` space-blanks fenced lines at the original byte length so `--fix` heading offsets still index the source. Symbol readers and `--fix` `###` scans run on the blanked text.
- `check --spec NAME` is a clap flag merged with positional SPEC in `main`.
- `GenerationOutcome.skipped_no_files`; generate exits 1 when generated=0 and that list is non-empty.
- `scaffold --dir` is confined with `confined_generation_path`.

## From the change's testing.md

# Testing

Unit and integration tests for each P1, plus the existing `fix_`, `generate_`, `new_`, and `get_spec_symbols` regressions.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-097 | `deleting_verification_attempts_refuses_adopted_finalization`, `emptying_verification_attempts_refuses_adopted_finalization`, `verifying_change_refuses_to_recreate_a_missing_attempts_ledger` |
| REQ-change-098 | `canonical_definition_payload_folds_crlf_to_lf`, `canonical_definition_payload_preserves_a_lone_carriage_return`, `canonical_tasks_payload_folds_crlf_before_checkbox_rewrite` |
| REQ-change-099 | `bundled_lifecycle_limits_match_the_github_scripts_copy` |
| REQ-cli-args-017 | `check_accepts_repeatable_spec_flag`, `check_repeatable_spec_flag_filters_the_same_as_positional` |
| REQ-cli-011 | `check_accepts_repeatable_spec_flag`, `check_repeatable_spec_flag_filters_the_same_as_positional` |
| REQ-cmd-check-016 | `fix_ignores_fenced_exported_heading_before_the_table` |
| REQ-cmd-generate-002 | `generate_fails_closed_when_unspecced_modules_have_no_source_files` |
| REQ-cmd-new-001 | `new_auto_detects_single_source_file` |
| REQ-cmd-new-004 | `new_refuses_reserved_and_invalid_module_names` |
| REQ-cmd-scaffold-004 | `scaffold_dir_must_stay_beneath_the_project_root` |
| REQ-config-014 | `test_load_config_malformed_json_sets_load_error`, `test_load_config_malformed_v4_json_sets_load_error`, `malformed_json_config_refuses_rules_rehash_compact_and_view` |
| REQ-generator-005 | `confined_generation_path_rejects_escape_and_absolute`, `generate_fails_closed_when_unspecced_modules_have_no_source_files` |
| REQ-git-utils-005 | `git_inherited_env_excludes_secrets_and_git_overrides`, `git_ceiling_stops_walk_into_a_host_worktree` |
| REQ-parser-004 | `fenced_example_backticks_are_not_documented_exports`, `fenced_exported_heading_is_not_a_public_api_subsection` |
| REQ-mcp-008 | `git_inherited_env_excludes_secrets_and_git_overrides`, `git_ceiling_stops_walk_into_a_host_worktree` |

## Where these lessons go

- `specs/change/context.md`
- `specs/cli_args/context.md`
- `specs/cmd_check/context.md`
- `specs/cmd_generate/context.md`
- `specs/cmd_new/context.md`
- `specs/cmd_scaffold/context.md`
- `specs/config/context.md`
- `specs/generator/context.md`
- `specs/git_utils/context.md`
- `specs/cli/context.md`
- `specs/parser/context.md`
- `specs/mcp/context.md`
