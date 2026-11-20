# Lesson bundle: route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Route and mark the listing and spawn sites the 6.4.1 scans found outside the hardening change's paths
- **Kind**: BugFix
- **Specs**: agents, cmd_comment, cmd_import, cmd_merge, cmd_migrate, config, generator, github, importer, mcp, watch, change, validator, cmd_check
- **Paths**: specs/generator/generator.spec.md, src/agents.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/doctor.rs, src/change/historical_definition_digest_matches.rs, src/change/spec_history.rs, src/commands/comment.rs, src/commands/import.rs, src/commands/merge.rs, src/commands/migrate.rs, src/config.rs, src/generator.rs, src/github.rs, src/importer.rs, src/mcp/preflight.rs, src/validator/consumed_by_re.rs, src/watch.rs, src/commands/check.rs
- **Acceptance**: The read, write, Git and spawn scans of REQ-util-008 pass with every site in these 17 files routed through the confined listing (REQ-util-010) or marked with its reason; scaffold --template reads through read_typed_file (REQ-util-011, REQ-cmd-scaffold-009); change audit covers every changed path; no requirement text changes.

## Evidence

- Verification commit: `eac242876ca3c73c12f918c90794e82aeee08cd0`
- Base commit: `6d155ce583e44b2411db84ec2ed307b1ed62bbdb`
- Verified by: `specsync check --spec agents --spec change --spec cmd_check --spec cmd_comment --spec cmd_import --spec cmd_merge --spec cmd_migrate --spec config --spec generator --spec github --spec importer --spec mcp --spec validator --spec watch`

## From the change's context.md

# Context

The 6.4.1 hardening change (`close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first`) widened the read, Git and spawn scans (X1) and routed SpecSync's record-folder listings through the confined listing (W1). Doing so touched 17 files outside its approved path list. SpecSync cannot add paths to an approved change without a delta for each owning module, so Leif chose a second change in the same PR that covers just those files, with no wording ('Yes, two changes (Recommended)', about 21:52Z on 2026-11-20, relayed by t4c). One review covers both changes.

sec-review2's re-check of 6.4.1 asked for C5's warning in `check`, one line in `src/commands/check.rs`, which neither change covered. The change was sent back and that path and its owner `cmd_check` were added, with no wording change, under Leif's standing OK for scope approvals ('Standing OK, tell me after', relayed by t4c).

## From the change's design.md

# Design

- **Record folders** (doctor's archive and closed-late listings, `spec_history`'s `history/`) are listed through `util::read_project_dir_entries`.
- **`agents`, `config`, `migrate` and the MCP preflight** list project folders through it too.
- **Every other listing or walk** keeps its code and carries a `not a project file read:` marker with its reason: walks with `follow_links(false)`, the confined handle in `consumed_by_re`, the `--from-dir` folder in `import`, and the archived package SpecSync just moved.
- **Spawned `gh`, `date` and the `specsync` rerun in `watch`** carry `spawned command writes no project file:`. `merge`'s Git `config` calls carry `git writes no project file:`.
- **`generator`** reads `--template` files through `read_typed_file`, and `check_template_dir` is documented in its spec.

## From the change's testing.md

# Testing

| Case | Test |
|------|------|
| No unmarked listing | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |
| No unmarked Git `config` call | `tests/integration/project_writes.rs`: `no_spawned_git_command_writes_files_unmarked` |
| No unmarked spawn | `tests/integration/project_writes.rs`: `no_spawned_command_other_than_git_is_unmarked` |
| `--template` FIFO | `tests/integration/hardening_641.rs`: `scaffold_does_not_wait_on_a_fifo_template` |

## Where these lessons go

- `specs/agents/agents.3md`
- `specs/cmd_comment/cmd_comment.3md`
- `specs/cmd_import/cmd_import.3md`
- `specs/cmd_merge/cmd_merge.3md`
- `specs/cmd_migrate/cmd_migrate.3md`
- `specs/config/config.3md`
- `specs/generator/generator.3md`
- `specs/github/github.3md`
- `specs/importer/importer.3md`
- `specs/mcp/mcp.3md`
- `specs/watch/watch.3md`
- `specs/change/change.3md`
- `specs/validator/validator.3md`
- `specs/cmd_check/cmd_check.3md`
