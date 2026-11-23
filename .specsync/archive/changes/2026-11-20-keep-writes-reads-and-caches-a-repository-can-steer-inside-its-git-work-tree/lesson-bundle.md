# Lesson bundle: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep writes, reads and caches a repository can steer inside its Git work tree
- **Kind**: BugFix
- **Specs**: agents, archive, change, changelog, cmd_check, cmd_generate, cmd_import, cmd_init_registry, cmd_init, cmd_issues, cmd_lifecycle, cmd_merge, cmd_migrate, cmd_view, commands, compact, config, deps, generator, git_utils, hash_cache, hooks, mcp, merge, rehash, util, validator
- **Paths**: src/agents.rs, src/archive.rs, src/change/collect_requirement_ids.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/doctor.rs, src/change/historical_definition_digest_matches.rs, src/change/read_cache.rs, src/change/retirement.rs, src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/verification_ignored_paths.rs, src/change_tests/doctor_collision.rs, src/changelog.rs, src/commands/check.rs, src/commands/generate.rs, src/commands/import.rs, src/commands/init.rs, src/commands/init_registry.rs, src/commands/issues.rs, src/commands/lifecycle.rs, src/commands/merge.rs, src/commands/migrate.rs, src/commands/mod.rs, src/commands/rehash.rs, src/commands/view.rs, src/compact.rs, src/config.rs, src/deps.rs, src/generator.rs, src/git_utils.rs, src/hash_cache.rs, src/hooks.rs, src/mcp/confinement.rs, src/mcp/protocol.rs, src/mcp/snapshot.rs, src/mcp/tools.rs, src/merge.rs, src/merge_driver.rs, src/util.rs, src/validator/consumed_by_re.rs, src/validator/tests.rs, specs/agents/, specs/archive/, specs/change/, specs/changelog/, specs/cmd_check/, specs/cmd_generate/, specs/cmd_import/, specs/cmd_init_registry/, specs/cmd_init/, specs/cmd_issues/, specs/cmd_lifecycle/, specs/cmd_merge/, specs/cmd_migrate/, specs/cmd_view/, specs/commands/, specs/compact/, specs/config/, specs/deps/, specs/generator/, specs/git_utils/, specs/hash_cache/, specs/hooks/, specs/mcp/, specs/merge/, specs/rehash/, specs/util/, specs/validator/, tests/integration.rs, tests/integration/, hi/check.md, INTENT.md, CHANGELOG.md
- **Acceptance**: Every change-workflow, backup, registry, init, lifecycle, merge and generator write goes through the confined project helpers: a committed change.md or artifact link out, a dangling artifact link, a linked .specsync/changes, a linked .specsync/backup-fix and a linked .specsync under init-registry are refused with the path named, and nothing outside is written. Inside a Git work tree the helpers confine to the work tree's top folder, so a monorepo subproject's links that stay in the repository work; absolute links are refused as absolute links; a path into the Git directory (a .git component in any case, trailing dots or spaces stripped, or a link whose canonical target lies under git rev-parse --absolute-git-dir or a .git component) is refused as leading into the Git directory; a write opens without blocking and checks the handle is a regular file before truncating. A .specsync/hashes.json Git tracks is neither read nor saved: check validates every spec with one warning naming git rm --cached, rehash exits 1, and prune drops keys that leave the project. specs_dir, source_dirs and schema_dir must be relative paths inside the project without ..; spec discovery walks the confined handle within 100,000 entries and 256 levels; changelog skips a spec blob over 8 MiB at a ref with one warning; import --from-dir reads within the named directory with the 8 MiB bound. A source scan fails on any new direct write, rename or removal outside the helpers, and the read scan also counts fs::copy, OpenOptions reads, BufReader and io::copy. The new tests fail on the base with only the tests added and pass with the fix

## Evidence

- Verification commit: `e843776af1169b66c685462d0da85972a0079f46`
- Base commit: `1b3974e9a025f37ce712d4ad627e99f780d6f207`
- Verified by: `specsync check --spec agents --spec archive --spec change --spec changelog --spec cmd_check --spec cmd_generate --spec cmd_import --spec cmd_init --spec cmd_init_registry --spec cmd_issues --spec cmd_lifecycle --spec cmd_merge --spec cmd_migrate --spec cmd_view --spec commands --spec compact --spec config --spec deps --spec generator --spec git_utils --spec hash_cache --spec hooks --spec mcp --spec merge --spec rehash --spec util --spec validator`

## From the change's context.md

# Context

An independent review of the private advisory branch at `0d9cbde5` found nine more paths a cloned repository can steer (F2 to F9), two regressions of the earlier fixes (R1, R2), and a gap in the guard (G1):

- **F2:** `change answer`, `depend`, `supersede`, `approve` and `start` wrote `change.md` and artifacts through committed links, and `change new` built its workspace in a linked `.specsync/changes`.
- **F3:** `check --fix --backup` copied specs through a linked `.specsync/backup-fix`.
- **F4:** a committed `.specsync/hashes.json` replayed a forged pass in `check`.
- **F5:** `specs_dir` was not confined, so the spec walk could run over `/`.
- **F6:** `changelog` kept a `git show` blob of any size.
- **F7:** `import --from-dir` read through links with no bound.
- **F8:** `init-registry` created its file in a linked `.specsync`.
- **F9:** `write_project_file` opened a file to write it without the non-blocking open and type check the read has.
- **R1:** links that stay inside the repository but leave `--root` were refused, which broke monorepo subprojects.
- **R2:** absolute links were refused with a misleading message.
- **G1:** nothing caught a new direct write, and the read scan missed `fs::copy`, `OpenOptions` reads and `BufReader`.

While building this, the builder also found that a link that stays inside the clone can lead into `.git`. With `change answer`, a committed `change.md -> ../../../.git/hooks/pre-commit` replaced the hook with text the repository chose.

Human decisions, relayed by the planning session (hc t4c):
- "yes" to the five points of the review (change 5).
- "yes" (2026-11-19, about 16:00Z) to CHECK-1.f as written and to choices 1 to 8 as recommended in `security/change5-scope.md`:
  1. a tracked `hashes.json` is neither read nor saved, with one warning and the `git rm --cached` fix;
  2. the confinement root is the Git work tree's top folder, else `--root`; the caches stay strict; absolute links are refused with an "absolute link" message;
  3. `specs_dir`, `source_dirs` and `schema_dir` are confined, with a budgeted confined walk;
  4. a changelog blob over 8 MiB is skipped with a warning;
  5. `import --from-dir` is confined to the named directory, with the 8 MiB bound;
  6. `--fix --backup` keeps its layout and writes confined;
  7. the wider write scan;
  8. the two low items (`prune`, and the empty-directory cleanup in `agents`) are included.
- "All as recommended" (about 19:43Z) to point 6: the project helpers refuse a path that resolves into the Git directory, directly or through links (a `.git` component in any case with trailing dots and spaces, or a canonical target under `git rev-parse --absolute-git-dir`), with `<path> leads into the Git directory`. SpecSync's own writes into `.git` keep their own paths.

Change 4 (F1) shipped first, on the same private PR. The fix ships in 6.4.0 with the dry run (#1100), after a second independent review.

## From the change's design.md

# Design

## The confinement (`util`)

- **Where the handle sits.** `confinement(canonical_root)` asks `git rev-parse --show-toplevel --absolute-git-dir` once per root, with `GIT_DIR` and `GIT_WORK_TREE` removed, and caches the answer for the run.
  - Inside a work tree, every project path is opened beneath the top folder through one `cap_std::Dir`, and `prefix` is the root's path beneath it. Messages strip `prefix`, so they still name paths relative to the root.
  - Outside a work tree, or without Git, the top is the root itself.
- **`inspect_project_path`** walks the path one component at a time and refuses:
  - an absolute link target, as "is an absolute link, which is never followed";
  - a `.git` component in any letter case with trailing dots and spaces trimmed;
  - a path or a link's canonical target under the Git directory or a `.git` component, as "leads into the Git directory";
  - a link out, a dangling link, and anything that is not a regular file or directory, as before.
- **Writing an existing file** opens it non-blocking and without truncating, checks the handle is a regular file, then calls `set_len(0)` and writes (F9).
- **New helpers:**
  - `read_file_within(directory, path, max)` reads within exactly a directory the person names;
  - `create_new_project_dir` creates a directory that must not exist yet;
  - `is_project_file` says whether a path is a regular file the functions would read;
  - `remove_empty_project_dir` removes an empty directory without following a link;
  - `find_project_files(root, directory, suffix, max_entries, max_depth)` walks through the handle. It lists file links for the reader to refuse, does not walk directory links or the Git directory, and stops past either budget. When the directory is the root itself, it starts at the handle's own directory.

## Writers routed through the helpers

- **The change workflow (F2):**
  - `write_change_markdown`, `ensure_artifact_files` (exclusive create), `allocate_change_workspace` and `create_change` use the helpers. `write_record_json` replaces the production `write_json`.
  - `ensure_change_files_writable` refuses a change whose `state.json`, `change.md`, `approvals.json` or selected artifact is refused, before `answer`, `depend`, `supersede`, `approve` or `start` writes anything.
  - The durable transaction writers refuse every link component.
- **Other writers:**
  - `check --fix --backup` writes its backups and `--fix` writes its specs through the helpers (F3);
  - `init-registry` creates its file through them (F8);
  - `init`, `lifecycle`, `merge`, `generator`, `doctor`, `agents` and `retirement` write through them too.
- **Marked exceptions.** A write that is not a project file write carries a `not a project file write:` marker on its statement. Examples are a temporary directory, the Git hooks directory and the merge driver's `%A` file.

## Inputs a repository supplies

- **The hash cache (F4).** `tracked_cache_refusal` asks Git only when `.specsync/hashes.json` exists. If Git tracks it, `load` returns an empty cache, `save` refuses, and `check` prints `warning: hash cache not used:` with the `git rm --cached` fix. `prune` keeps only keys `confine_path_to_root` accepts that name a project file.
- **Configured directories (F5).** `validate_config_directories` runs in the ambient TOML and JSON loaders. The MCP snapshot check applies it per request. `check` and `issues` keep their own confined handling.
- **Spec discovery (F5).** `find_spec_files_checked` uses `find_project_files` with 100,000 entries and 256 levels. `find_spec_files` prints the reason and exits 1.
- **The changelog (F6).** `read_file_at_ref` asks `git cat-file -s` first and reads with `output_after_child_exits_capped`. A blob over the spec bound is skipped with one warning.
- **Imports (F7).** `read_import_file` reads a project file by the project rule. A file under an outside directory is read with `read_file_within` that directory.

## The scans (G1)

- **`no_project_file_is_written_outside_the_confined_helpers`** fails on a direct write, copy, create, rename, `persist`, removal or `set_permissions` outside the helpers. A statement-scoped marker excuses one.
- **The read scan** now also counts `fs::copy`, `.read(true)`, `BufReader::new` and `io::copy`.
- **Both scans** skip `#[cfg(test)]` blocks. Every remaining allow-listed file carries a reason.

## From the change's testing.md

# Testing

Fixtures are temporary directories, most with their own `git init`. Every link points inside the fixture's own temporary directory, and "outside" is a sibling directory there.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-004 | `tests/integration/project_writes.rs`: `a_subproject_writes_through_a_link_that_stays_inside_its_repository`, `a_subproject_reads_specs_through_a_link_that_stays_inside_its_repository`, `a_subproject_link_that_leaves_its_repository_is_still_refused`, `an_absolute_link_is_refused_as_an_absolute_link`; `src/util.rs`: `writing_a_fifo_is_refused_without_blocking`, `reading_within_a_named_directory_stays_inside_it`, `removing_an_empty_directory_never_follows_a_link_out`, `finding_project_files_never_follows_a_link`, `finding_project_files_from_the_root_itself_lists_its_files` |
| REQ-util-005 | `tests/integration/project_writes.rs`: `hooks_install_never_writes_through_a_link_into_the_git_directory`, `change_answer_never_writes_change_md_into_a_hook`; `src/util.rs`: `project_file_functions_refuse_the_git_directory`, `project_file_functions_refuse_a_git_directory_under_any_name` |
| REQ-change-163 | `tests/integration/project_writes.rs`: `change_answer_never_writes_change_md_through_a_link_out`, `change_approve_never_writes_change_md_through_a_link_out`, `change_answer_never_creates_the_target_of_a_dangling_artifact_link`, `change_new_creates_nothing_in_a_changes_directory_linked_out`, `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-check-020 | `tests/integration/project_writes.rs`: `check_fix_backup_copies_nothing_through_a_linked_backup_directory`, `check_fix_backup_never_copies_over_a_linked_backup_file`, `check_fix_backup_still_backs_up_a_project_with_no_link` |
| REQ-cmd-check-021, REQ-hash-cache-009 | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_git_tracks`, `rehash_refuses_a_hash_cache_git_tracks`; `src/hash_cache.rs`: `a_forged_pass_in_a_cache_git_tracks_is_never_replayed` |
| REQ-hash-cache-010 | `src/hash_cache.rs`: `prune_drops_keys_that_leave_the_project` |
| REQ-config-017 | `tests/integration/repository_inputs.rs`: `a_configured_directory_outside_the_project_is_a_config_error`; `tests/integration/mcp.rs`: `mcp_rejects_configured_read_and_write_path_escapes` |
| REQ-validator-050 | `tests/integration/repository_inputs.rs`: `a_specs_link_that_leaves_the_project_is_not_walked`, `spec_discovery_stops_at_the_depth_budget`; `src/validator/tests.rs`: `test_find_spec_files_empty_dir`, `test_find_spec_files_with_specs` |
| REQ-validator-051 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |
| REQ-changelog-002, REQ-git-utils-012, REQ-git-utils-013 | `tests/integration/repository_inputs.rs`: `changelog_skips_a_spec_blob_over_the_bound`; `src/git_utils.rs`: `capped_output_keeps_the_cap_and_reports_more` |
| REQ-cmd-import-004 | `tests/integration/repository_inputs.rs`: `import_from_dir_never_imports_through_a_link_out`, `import_from_an_outside_dir_stays_inside_that_dir` |
| REQ-cmd-init-registry-004 | `tests/integration/project_writes.rs`: `init_registry_creates_nothing_in_a_specsync_directory_linked_out` |
| REQ-agents-013 | `src/util.rs`: `removing_an_empty_directory_never_follows_a_link_out`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-archive-003 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-compact-003 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-merge-008 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-merge-006 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-lifecycle-006 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-init-010 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-generator-012 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-generate-005 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-issues-004 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-hooks-008 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`; `hooks_install_never_writes_through_a_link_into_the_git_directory` |
| REQ-mcp-013 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`; `tests/integration/mcp.rs`: `mcp_rejects_configured_read_and_write_path_escapes` |
| REQ-commands-015 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-rehash-002 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-view-003 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-deps-005 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-migrate-007 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |

Red first on `1b3974e9` with only the tests commit added (`dfdaf4f6` in the builder's clone, `fc40682d` here): 22 of the 24 new integration tests and both new `hash_cache` unit tests failed. The write scan found 117 direct writes and the read scan 27 direct reads outside the helpers. The two controls passed there, as they should: a subproject link that leaves the repository is still refused, and a backup with no link still works. The new `src/util.rs` unit tests call helpers this change adds, so they could not run on the base; the Git-directory and walk cases they cover are also driven through the CLI by `hooks_install_never_writes_through_a_link_into_the_git_directory`, `change_answer_never_writes_change_md_into_a_hook` and `a_specs_link_that_leaves_the_project_is_not_walked`.

Then the full verify lane and the Linux container run.

## Where these lessons go

- `specs/agents/agents.3md`
- `specs/archive/archive.3md`
- `specs/change/change.3md`
- `specs/changelog/changelog.3md`
- `specs/cmd_check/cmd_check.3md`
- `specs/cmd_generate/cmd_generate.3md`
- `specs/cmd_import/cmd_import.3md`
- `specs/cmd_init_registry/cmd_init_registry.3md`
- `specs/cmd_init/cmd_init.3md`
- `specs/cmd_issues/cmd_issues.3md`
- `specs/cmd_lifecycle/cmd_lifecycle.3md`
- `specs/cmd_merge/cmd_merge.3md`
- `specs/cmd_migrate/cmd_migrate.3md`
- `specs/cmd_view/cmd_view.3md`
- `specs/commands/commands.3md`
- `specs/compact/compact.3md`
- `specs/config/config.3md`
- `specs/deps/deps.3md`
- `specs/generator/generator.3md`
- `specs/git_utils/git_utils.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/hooks/hooks.3md`
- `specs/mcp/mcp.3md`
- `specs/merge/merge.3md`
- `specs/rehash/rehash.3md`
- `specs/util/util.3md`
- `specs/validator/validator.3md`
