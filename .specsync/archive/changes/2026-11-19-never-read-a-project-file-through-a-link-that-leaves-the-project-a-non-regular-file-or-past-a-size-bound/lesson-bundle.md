# Lesson bundle: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Never read a project file through a link that leaves the project, a non-regular file, or past a size bound
- **Kind**: BugFix
- **Specs**: util, agents, archive, change, cmd_change, cmd_check, cmd_diff, cmd_init, cmd_lifecycle, cmd_merge, cmd_migrate, commands, cmd_report, cmd_resolve, cmd_stale, compact, config, deps, exports, generator, hash_cache, hooks, ignore, mcp, merge, output, registry, schema, scoring, validator, view
- **Paths**: specs/util/, specs/agents/, specs/archive/, specs/change/, specs/cmd_change/, specs/cmd_check/, specs/cmd_diff/, specs/cmd_init/, specs/cmd_lifecycle/, specs/cmd_merge/, specs/cmd_migrate/, specs/commands/, specs/cmd_report/, specs/cmd_resolve/, specs/cmd_stale/, specs/compact/, specs/config/, specs/deps/, specs/exports/, specs/generator/, specs/hash_cache/, specs/hooks/, specs/ignore/, specs/mcp/, specs/merge/, specs/output/, specs/registry/, specs/schema/, specs/scoring/, specs/validator/, specs/view/, src/util.rs, src/agents.rs, src/archive.rs, src/change.rs, src/change/, src/change_tests.rs, src/change_tests/, src/commands/change.rs, src/commands/check.rs, src/commands/diff.rs, src/commands/init.rs, src/commands/lifecycle.rs, src/commands/merge.rs, src/commands/migrate.rs, src/commands/mod.rs, src/commands/report.rs, src/commands/resolve.rs, src/commands/stale.rs, src/compact.rs, src/config.rs, src/deps.rs, src/exports/, src/generator.rs, src/hash_cache.rs, src/hooks.rs, src/ignore.rs, src/mcp_tools_lock.rs, src/merge.rs, src/output.rs, src/registry.rs, src/schema.rs, src/scoring.rs, src/validator/, src/validator.rs, src/view.rs, tests/integration.rs, tests/integration/, hi/setup.md, INTENT.md, CHANGELOG.md, site/src/content/docs/cli.md
- **Acceptance**: With .specsync/sdd.json, .specsync/change-transaction.json, a config file, a change record, a spec, a companion, a source or schema file, .specsync/hashes.json or another project file SpecSync reads: a symbolic link to a file outside the project, a dangling link, a FIFO, or a regular file larger than its bound is not read, the command does not wait on it, and the read fails the way that read's other read errors do today with a message naming the path and the reason; a link to a regular file inside the project is read as before; the bounds are 4 MiB for policy, config, state and change records, 8 MiB for specs, companions, templates, hook files, source and schema files, and 64 MiB for the transaction journal and the hash cache; each new test fails on 3a02a5d5 and passes with the fix

## Evidence

- Verification commit: `0d9cbde58d6637dc8192c627291dfbe9eb7efb04`
- Base commit: `3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef`
- Verified by: `specsync check --spec agents --spec archive --spec change --spec cmd_change --spec cmd_check --spec cmd_diff --spec cmd_init --spec cmd_lifecycle --spec cmd_merge --spec cmd_migrate --spec cmd_report --spec cmd_resolve --spec cmd_stale --spec commands --spec compact --spec config --spec deps --spec exports --spec generator --spec hash_cache --spec hooks --spec ignore --spec mcp --spec merge --spec output --spec registry --spec schema --spec scoring --spec util --spec validator --spec view`

## From the change's context.md

# Context

A tester found that SpecSync 6.3.0 reads `.specsync/sdd.json` through a symbolic link with no size bound. A committed link to `/dev/zero` made `specsync change status --json` reach 227 GB on macOS and get killed by the OOM killer on Linux, and Chough's `next` and `status` call that command. `.specsync/change-transaction.json` is read the same way. A FIFO at either path makes the command wait forever. On this branch before the change, a link from `sdd.json` to a file outside the project was read, a FIFO blocked `change status --json` past 8 seconds, and a 5 MiB file was read whole.

The same shape is in about 140 reads: `std::fs::read` or `read_to_string` on a project path, which follows any link, waits on a FIFO and reads without a bound.

Human decisions, relayed by the planning session: "Yes, a 3rd change" (same advisory and release; every SpecSync read of a project file refuses a link that leaves the project and a non-regular file, with a size bound), then "Approve" on the written scope (2026-11-19, about 14:21Z).

What a session picking this up needs to know:

- It is the third change on the private advisory PR, after the hash cache and the project file writes.
- The approved scope, in short:
  - every read of a project file goes through one bounded helper on the project root, opened without blocking, type-checked on the handle, and read to at most the bound plus one byte;
  - a link that stays inside the project is followed; a link out, a dangling link, a non-regular file and a file over its bound are refused, naming the path and the reason;
  - a refused read is handled like that read's other read errors today;
  - the caches stay strict.
- The bounds:
  - 4 MiB for policy, config, state and change records;
  - 8 MiB for specs, companions, templates, hook files, source and schema files;
  - 64 MiB for the transaction journal and `hashes.json`.
- Out of scope:
  - paths named on the command line (`import --from-dir`, `change correct-owner --manifest`);
  - the repository's own `.git`, including the pre-commit hook in the Git hooks directory;
  - the files Git hands the merge driver;
  - Git object reads and MCP stdin, which are already bounded;
  - reads already inside the confined, bounded discovery.
- One read is not named in the scope's table: comparing a delivery file's digest when a change is reconciled. It gets the 64 MiB project input bound, since that is what it reads.
- Never use `/dev/zero` in a test: a run on the unfixed code would exhaust memory on a shared machine.

## From the change's design.md

# Design

**`util`.** It gains the bounded read, on the same `cap_std` handle as change 2's write functions.

- `read_project_file(root, path, max_bytes) -> io::Result<Option<Vec<u8>>>` takes the bound it was missing. It already opened without blocking and checked the file type on the handle. Now it also refuses a file whose length is over the bound, and it reads at most `max_bytes + 1` bytes, so a file that grows while it is read is refused too.
- `read_project_bytes` and `read_project_string` have the same contract as `std::fs::read` and `read_to_string`, a missing file is a not-found error, so each call site keeps its error handling.
- The refusal reasons are:
  - `<path> is a symbolic link that leaves the project`
  - `<path> is a dangling symbolic link`
  - `<path> is not a regular file`
  - `<path> is larger than <n> bytes`
- The bounds are named constants: `MAX_STATE_FILE_BYTES` (4 MiB), `MAX_SPEC_FILE_BYTES` and `MAX_SOURCE_FILE_BYTES` (8 MiB), and `MAX_PROJECT_INPUT_BYTES` (64 MiB).

**Call sites.** Each `fs::read`, `fs::read_to_string` or `File::open` of a project path becomes the matching `util` read with its bound, and keeps its existing error handling.
- Functions that had only a path gain the project root.
- `hash_file(root, path)` reads up to 8 MiB and hashes it. A refused or missing file is unreadable, which the cache already treats as changed.
- `HashCache::read` keeps its strict no-link handle and adds the 64 MiB bound.
- The remote spec cache keeps its strict check and reads with the 8 MiB bound.

**The guard.** An integration test scans every non-test source file for direct `fs::read`, `fs::read_to_string`, `File::open` and `read_to_end`, outside an allow-list:
- `util` itself;
- the confined readers that are already bounded;
- Git output;
- the merge-driver files;
- the paths named on the command line;
- the repository's own `.git`.

A new direct read of a project file fails it.

**Not changed:** the reads already inside the confined, bounded discovery.

## From the change's testing.md

# Testing

Every fixture is a temporary directory holding the project and an `outside` directory, so no link reaches a real file. FIFO tests run the command under a timeout, so a run on the unfixed code fails instead of hanging. Over-bound files are sparse, so they cost no disk. Nothing reads `/dev/zero`. The link and FIFO tests are Unix-only.

| Test | Shows | On `3a02a5d5` |
|---|---|---|
| `src/util.rs` `project_file_reads_*` | the bounded read: link out, dangling, FIFO, directory, over-bound, at-bound, inside link, missing | does not compile (the bound and the two functions are new) |
| `tests/integration/project_reads.rs` FIFO cases | each command finishes without waiting | times out |
| the same file, outside-link cases | the outside bytes are not used, and the message names the link | the outside file is read |
| the same file, over-bound cases | the read is refused naming the bound | the file is read whole |
| `no_project_file_is_read_outside_the_bounded_read` | no direct project file read remains | about 140 sites found |

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-003 | `src/util.rs`: `project_file_reads_refuse_what_they_must_not_read`, `project_file_reads_hold_the_bound`, `project_file_reads_follow_inside_links_and_keep_the_std_contracts`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-change-161 | `tests/integration/project_reads.rs`: `change_status_does_not_wait_on_a_policy_that_is_not_a_regular_file`, `change_status_does_not_read_a_policy_linked_outside_the_project`, `change_status_refuses_a_policy_over_its_bound`, `a_lifecycle_write_does_not_wait_on_or_follow_the_transaction_journal`, `a_policy_linked_inside_the_project_is_still_read`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-cmd-change-038 | `change_audit_does_not_wait_on_the_policy`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-config-016 | `check_does_not_wait_on_a_config_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-ignore-002 | `check_does_not_wait_on_an_ignore_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-hash-cache-008 | `check_does_not_wait_on_a_companion_that_is_not_a_regular_file`, `check_refuses_a_hash_cache_over_its_bound`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-validator-049 | `check_does_not_read_requirements_linked_outside_the_project`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-exports-011 | `check_does_not_read_exports_from_a_source_linked_outside_the_project`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-mcp-012 | `mcp_diff_does_not_wait_on_a_lock_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-agents-012 | `agents_install_does_not_wait_on_a_manifest_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-archive-002 | `archive_tasks_does_not_wait_on_a_tasks_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-hooks-007 | `hooks_status_does_not_wait_on_a_hook_file_that_is_not_a_regular_file`; the source scan `no_project_file_is_read_outside_the_bounded_read` |
| REQ-cmd-check-019 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-diff-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-init-009 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-lifecycle-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-merge-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-migrate-006 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-commands-014 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-report-006 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-resolve-003 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-cmd-stale-005 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-compact-002 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-deps-004 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-generator-011 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-merge-007 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-output-007 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-registry-004 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-schema-002 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-scoring-007 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |
| REQ-view-002 | the source scan `no_project_file_is_read_outside_the_bounded_read` (no direct read remains in the module) and the shared `util` tests |

Red first on `3a02a5d5`, with only the tests added: 11 of the 17 integration tests failed, among them the source scan with 187 direct reads. The 6 that passed are guards that already held on base: the confined discovery for a FIFO companion and for linked-out source and requirements files, `archive-tasks`' own preflight, change 2's agents manifest read, and the inside-link control. The `util` unit tests use the new bound and functions, so they could not run on base.

Then the full `fledge lanes run verify` lane locally, and `cargo test` in a local Linux container, since the private fork runs no CI.

## Where these lessons go

- `specs/util/util.3md`
- `specs/agents/agents.3md`
- `specs/archive/archive.3md`
- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cmd_check/cmd_check.3md`
- `specs/cmd_diff/cmd_diff.3md`
- `specs/cmd_init/cmd_init.3md`
- `specs/cmd_lifecycle/cmd_lifecycle.3md`
- `specs/cmd_merge/cmd_merge.3md`
- `specs/cmd_migrate/cmd_migrate.3md`
- `specs/commands/commands.3md`
- `specs/cmd_report/cmd_report.3md`
- `specs/cmd_resolve/cmd_resolve.3md`
- `specs/cmd_stale/cmd_stale.3md`
- `specs/compact/compact.3md`
- `specs/config/config.3md`
- `specs/deps/deps.3md`
- `specs/exports/exports.3md`
- `specs/generator/generator.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/hooks/hooks.3md`
- `specs/ignore/ignore.3md`
- `specs/mcp/mcp.3md`
- `specs/merge/merge.3md`
- `specs/output/output.3md`
- `specs/registry/registry.3md`
- `specs/schema/schema.3md`
- `specs/scoring/scoring.3md`
- `specs/validator/validator.3md`
- `specs/view/view.3md`
