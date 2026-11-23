# Lesson bundle: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Walk configured folders only through the confined walk, and keep the read cache in the user cache folder
- **Kind**: BugFix
- **Specs**: util, change, validator, config, schema, commands, cmd_scaffold, registry, cmd_wizard, mcp, cmd_check
- **Paths**: src/util.rs, src/util/local_state.rs, src/change/read_cache.rs, src/change/collect_requirement_ids.rs, src/change_tests/fast_archived_reads.rs, src/change_tests/journal_replay_targets.rs, src/validator/requirement_ids.rs, src/validator/consumed_by_re.rs, src/validator/tests.rs, src/config.rs, src/schema.rs, src/commands/mod.rs, src/commands/scaffold.rs, src/registry.rs, src/commands/wizard.rs, src/mcp/tools.rs, src/commands/check.rs, tests/integration
- **Acceptance**: Every walk or listing of a configured folder (specs_dir, source_dirs, schema_dir) goes through util's confined walk (find_project_files, read_project_dir), so a committed link that leaves the project at the folder or on the way to it is refused with the path instead of walked: change adopt's requirement proposals, the duplicate requirement ID check, init-registry's registry, the MCP issues walk, the wizard's source walk, the schema snapshot and check's schema inputs, scaffold's case-collision listing, a missing file's suggestion, and source auto-detection, which skips linked folders. The change read cache (archived history verdicts) lives in specsync/repositories/<key> in the user's cache folder, keyed by the canonical Git common directory so worktrees still share it, never in the Git directory, and is refused like the other state when the cache folder lies inside the project. The new tests fail on 4516f355 with only the tests added and pass with the fix

## Evidence

- Verification commit: `3c466fd114df448dcb4fac7e1eeab314d5666b7b`
- Base commit: `4516f355b3ad118e90ce86b7d36d4bd5e4a4c14b`
- Verified by: `specsync check --spec change --spec cmd_check --spec cmd_scaffold --spec cmd_wizard --spec commands --spec config --spec mcp --spec registry --spec schema --spec util --spec validator`

## From the change's context.md

# Context

While change 7 was being built, two more gaps of the same classes were found and put to Leif by t4c. He answered "continue with 95% confidence" (2026-11-20, about 04:09Z), which t4c read as yes to both. Under Leif's delegation of 2026-11-20, t4c decided they go into a change 8 on top of change 7, with no send-back.

- **Configured-folder walks.** About nine walks start with `walkdir` or `read_dir` at a configured folder, and `walkdir` follows a link at its root. So a committed `specs`, source or schema link that leaves the project is walked:
  - `change adopt`'s requirement proposals counted two `requirements.md` files outside the project;
  - the duplicate requirement ID check;
  - `init-registry`'s registry;
  - the MCP issues walk;
  - the wizard's source walk;
  - the schema snapshot and `check`'s schema inputs;
  - `scaffold`'s case-collision listing;
  - a missing file's suggestion;
  - source auto-detection.

  File reads were already bounded and refused links out, but the walks themselves could run over the disk and show names or counts.
- **The read cache.** `src/change/read_cache.rs` keeps passing verdicts for archived change history in the Git common directory. An archive that ships `.git` could plant them.

## From the change's design.md

# Design

- **`util::read_project_dir(root, path)`** is a one-level listing through the project rule. It is used by the schema snapshot, `check`'s schema inputs, `scaffold`'s case-collision check (`check_case_collision` now takes the root) and `suggest_similar_file`.
- **`validator::find_files_named(root, dir, name)`** is the confined, budgeted walk for exact names. It is used by `change adopt`'s requirement proposals and the duplicate-ID check.
- **`find_spec_files_checked`** is used by `generate_registry` and the MCP issues walk.
- **`find_project_files`** is used by the wizard's source walk.
- **Source auto-detection** skips a child of the root that is a symbolic link, and walks with `follow_root_links(false)`.
- **`local_state::repository_state_dir(root, common_dir)`** gives `<user cache>/specsync/repositories/<key>`, where the key is a digest of the canonical common directory. A cache folder inside the project is refused, as for the other state. `persistent_read_cache_path` returns that folder joined with `read-cache-v1.json`.

## From the change's testing.md

# Testing

Fixtures are temporary directories, with "outside" a sibling folder. Integration tests set `XDG_CACHE_HOME` outside every fixture.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-009 | `src/change_tests/fast_archived_reads.rs`: `the_read_cache_never_lives_in_the_git_directory`, `a_second_worktree_reads_warm_from_the_shared_cache` |
| REQ-util-010 | `src/schema.rs`: `a_schema_folder_linked_out_of_the_project_is_never_listed`; `src/validator/tests.rs`: `a_missing_file_suggestion_never_lists_a_folder_outside_the_project`; `tests/integration/project_writes.rs`: `scaffold_never_lists_a_specs_folder_linked_out_of_the_project` |
| REQ-change-164 | `src/change_tests/journal_replay_targets.rs`: `requirement_id_proposals_never_walk_a_specs_link_that_leaves_the_project`; `src/change_tests/fast_archived_reads.rs`: `the_read_cache_never_lives_in_the_git_directory` |
| REQ-validator-053 | `src/validator/requirement_ids.rs`: `duplicate_requirement_ids_are_never_looked_for_through_a_specs_link_out`, `an_unreadable_specs_subdirectory_is_an_error_naming_the_path`; `src/validator/tests.rs`: `a_missing_file_suggestion_never_lists_a_folder_outside_the_project` |
| REQ-config-018 | `src/config.rs`: `source_detection_never_follows_a_linked_folder` |
| REQ-schema-003 | `src/schema.rs`: `a_schema_folder_linked_out_of_the_project_is_never_listed` |
| REQ-commands-016 | `src/commands/mod.rs`: `case_collision_detects_case_fold_duplicates`; `tests/integration/project_writes.rs`: `scaffold_never_lists_a_specs_folder_linked_out_of_the_project` |
| REQ-cmd-scaffold-008 | `tests/integration/project_writes.rs`: `scaffold_never_lists_a_specs_folder_linked_out_of_the_project` |
| REQ-registry-005 | `src/registry.rs`: the existing `generate_registry` tests, on the confined walk |
| REQ-cmd-wizard-004 | `src/commands/wizard.rs`: the existing wizard tests, on the confined walk |
| REQ-mcp-014 | `tests/integration/mcp.rs`: the existing `specsync_issues` tests, on the confined walk |
| REQ-cmd-check-022 | `tests/integration/config.rs` and `tests/integration/check.rs`: the schema-input cache tests, on the confined listing |

Red first on `4516f355` with only change 8's records and tests added (`38cc5e87`): all 6 new unit tests and the new scaffold CLI test failed. `change adopt`'s proposals listed the two `requirements.md` files outside, the duplicate-ID check named `specs/alpha` and `specs/beta`, source detection found `lib` outside, the schema snapshot named `outside_only.sql`, the suggestion listed `../outside/data`, `scaffold` refused `auth` because of `Auth` outside, and the read cache was under `.git`.

Then the full verify lane and the Linux container run.

## Where these lessons go

- `specs/util/util.3md`
- `specs/change/change.3md`
- `specs/validator/validator.3md`
- `specs/config/config.3md`
- `specs/schema/schema.3md`
- `specs/commands/commands.3md`
- `specs/cmd_scaffold/cmd_scaffold.3md`
- `specs/registry/registry.3md`
- `specs/cmd_wizard/cmd_wizard.3md`
- `specs/mcp/mcp.3md`
- `specs/cmd_check/cmd_check.3md`
