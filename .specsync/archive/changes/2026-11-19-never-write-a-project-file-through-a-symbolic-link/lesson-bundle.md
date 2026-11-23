# Lesson bundle: never-write-a-project-file-through-a-symbolic-link

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Never write a project file through a symbolic link
- **Kind**: BugFix
- **Specs**: util, hooks, agents, mcp, registry, cmd_resolve, cmd_migrate, generator, cmd_scaffold, cmd_new, cmd_wizard, cmd_import
- **Paths**: src/util.rs, src/hooks.rs, src/agents.rs, src/mcp_tools_lock.rs, src/registry.rs, src/commands/resolve.rs, src/commands/migrate.rs, src/generator.rs, src/commands/scaffold.rs, src/commands/new.rs, src/commands/wizard.rs, src/commands/import.rs, tests/integration.rs, tests/integration/, specs/util/, specs/hooks/, specs/agents/, specs/mcp/, specs/registry/, specs/cmd_resolve/, specs/cmd_migrate/, specs/generator/, specs/cmd_scaffold/, specs/cmd_new/, specs/cmd_wizard/, specs/cmd_import/, hi/setup.md, INTENT.md, CHANGELOG.md, site/src/content/docs/cli.md
- **Acceptance**: For each of hooks install and uninstall, agents install and uninstall, mcp lock --write, the scaffold registry append, migrate, new, scaffold, add-spec, wizard and import: with the file it writes, or a directory on the way to it, a symbolic link that leaves the project, a dangling link, or a non-regular file, the command refuses with an error naming the path, the link's target is unchanged and nothing is created outside the project; with a link to a regular file inside the project (CLAUDE.md to AGENTS.md) it writes through the link; the resolve --remote cache is never read or written through any link and says so once on stderr; each new test fails on 7b6e4779 and passes with the fix

## Evidence

- Verification commit: `b8e3558621e94aaa50cdd8f668b8cc7856fc60aa`
- Base commit: `7b6e4779ba5b7d0fedd621c04a917fcb02e28c25`
- Verified by: `specsync check --spec agents --spec cmd_import --spec cmd_migrate --spec cmd_new --spec cmd_resolve --spec cmd_scaffold --spec cmd_wizard --spec generator --spec hooks --spec mcp --spec registry --spec util`

## From the change's context.md

# Context

A private sweep followed GHSA-7q5v-8jcc-rx54 (the hash cache written through a committed symbolic link). It found the same shape in commands a person runs on purpose inside a repository: a write to a fixed project path with plain `root.join(...)` and `fs::write`, which follows a link the repository committed. None of them runs on a plain `check`, so each needs the person to run the command in a clone.

- `hooks install` and `uninstall`: `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md`, `.claude/settings.json`.
- `agents install` and `uninstall`: the generated skill and command files and `.specsync/agent-artifacts.json`. A dangling link passed the "is it there" read and was created through.
- `mcp lock --write`: `.specsync/mcp-tools.lock.json`.
- `scaffold`: the registry append.
- `resolve --remote`: the cache under `.specsync-cache/remote-specs/`, whose file name the repository's own `depends_on` chooses.
- `migrate`: every fixed path it writes, and the spec files it rewrites, found by a walker that follows links. `.specsync/.gitignore` was guarded by `exists()`, which a dangling link passes.
- Found while reading, confirmed on released 6.3.0: `new`, `scaffold` and `add-spec` create the target of a dangling `specs/<name>/<name>.spec.md` link, and write the spec and four companions into a directory `specs/<name>` links to. `import` and the wizard share that code.

Not affected, checked on 6.3.0: `lifecycle promote` and `check --fix` stop at discovery on a linked spec file.

Human decisions, relayed by the planning session: "Fold into the same private fix"; for the siblings, "Yes, fold them in"; for refusing with an error and skipping the remote cache with a warning, "OK as proposed"; and, asked about `CLAUDE.md` linked to `AGENTS.md`, "Allow links that stay inside". Intent: SETUP-6, "A command I run in a repository somebody else wrote never writes outside it through a link it found there."

What a session picking this up needs to know:

- It stays in the advisory's private fork with the first change, on the same pull request.
- The hash cache keeps its stricter rule from the first change: any link is refused. The remote spec cache gets that strict rule too.
- Ruled out: refusing every link. `CLAUDE.md` as a link to `AGENTS.md` is a common setup and must keep working.

## From the change's design.md

# Design

**Shared functions in `src/util.rs`.** Each takes the project root and a path that is relative to it or absolute beneath it.

- `read_project_file(root, path) -> io::Result<Option<Vec<u8>>>`: `None` when nothing is there.
- `write_project_file(root, path, contents) -> io::Result<()>`: creates missing directories, replaces the content of an existing regular file, creates a missing one.
- `create_project_file(root, path, contents) -> io::Result<bool>`: `false` when a regular file is already there, otherwise an exclusive create.
- `remove_project_file(root, path) -> io::Result<()>`.
- `project_path_refusal(root, path) -> Option<String>`: the reason a path is refused, for a preflight.

All of them open a `cap_std::fs::Dir` on the root and act through it, so a path cannot leave the root even if a component is swapped while they run. A link inside the root is followed. Refused with a typed error whose message names the path: a link that leaves the root, an absolute link target, `..`, a dangling link, a target that is not a regular file.

**Callers.**

- `hooks`: the markdown and settings writers and `remove_section_from_file` take the root and use the functions. A refusal is the `Err` they already return.
- `agents`: the inspect read, the artifact write, the manifest read and write, and the uninstall removal use them. Every artifact is inspected before the first write, so a refusal comes before any write.
- `mcp`: `cmd_lock_write` uses `write_project_file`.
- `registry`: `register_module` returns `Result<bool, String>`; `scaffold` prints the error and exits 1.
- `cmd_resolve`: the cache checks its directories and file for any link, skips the entry with one warning, and writes through the same functions.
- `cmd_migrate`: a preflight calls `project_path_refusal` on every path the steps touch and on every discovered spec file, and stops before the first step. The steps then write through the functions.
- `generator`: the companion functions take the root and call `create_project_file`. A refused companion is reported on stderr and the rest continue.
- `cmd_scaffold`, `cmd_new`, `cmd_wizard`, `cmd_import`: the spec file is created with `create_project_file`.

**Not changed.** `hash_cache` keeps its own stricter handle from the first change. `hooks::is_installed` still only reads.

## From the change's testing.md

# Testing

Every fixture is one temporary directory holding the project and an `outside` directory beside it, so no link in any test reaches a real file. Link tests are Unix-only.

For each write site the cases are: a link to a file outside the project, a dangling link, a link to a link that leaves, a linked directory on the way, and a link that stays inside. Each new test is run on `7b6e4779` first and its failure kept as evidence.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-002 | `src/util.rs`: the five `project_file_functions_*` tests |
| REQ-hooks-006 | `tests/integration/project_links.rs`: `hooks_install_refuses_a_target_linked_outside_the_project`, `hooks_install_refuses_a_dangling_link_and_a_chain_that_leaves`, `hooks_install_creates_nothing_in_a_directory_linked_outside_the_project`, `hooks_install_writes_through_a_link_that_stays_inside_the_project`, `hooks_uninstall_refuses_a_target_linked_outside_the_project` |
| REQ-agents-011 | `tests/integration/project_links.rs`: `agents_install_refuses_a_dangling_artifact_link_before_writing_anything`, `agents_install_refuses_a_manifest_linked_outside_the_project`, `agents_install_creates_nothing_in_a_directory_linked_outside_the_project` |
| REQ-mcp-011 | `tests/integration/project_links.rs`: `mcp_lock_write_refuses_a_lock_linked_outside_the_project`, `mcp_lock_write_writes_a_plain_lock_and_through_a_link_that_stays_inside` |
| REQ-registry-003 | `src/registry.rs`: `register_module_refuses_a_registry_linked_outside_the_project`, `register_module_appends_once_to_a_regular_registry` |
| REQ-cmd-resolve-002 | `src/commands/resolve.rs`: `spec_cache_never_writes_or_reads_through_a_link`, `spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory` |
| REQ-cmd-migrate-005 | `tests/integration/project_links.rs`: `migrate_refuses_a_dangling_gitignore_link_before_its_first_write`, `migrate_refuses_a_spec_linked_outside_the_project`, `migrate_creates_nothing_in_a_specsync_directory_linked_outside_the_project`, `migrate_still_migrates_a_project_with_no_link` |
| REQ-generator-010 | `src/generator.rs`: `companion_files_are_never_created_through_a_link_that_leaves_the_project`; `tests/integration/project_links.rs`: `scaffold_refuses_a_dangling_companion_link_and_creates_the_others` |
| REQ-cmd-scaffold-007 | `tests/integration/project_links.rs`: the three `spec_creation_*` tests (each runs `scaffold` and `add-spec`) and `scaffold_refuses_a_registry_linked_outside_the_project` |
| REQ-cmd-new-006 | `tests/integration/project_links.rs`: the three `spec_creation_*` tests (each runs `new`) |
| REQ-cmd-wizard-003 | `src/commands/wizard.rs`: `wizard_write_refuses_a_dangling_spec_link_and_a_linked_spec_directory` |
| REQ-cmd-import-003 | `src/commands/import.rs`: `import_item_refuses_a_dangling_spec_link_and_a_linked_spec_directory` |

Red first on `7b6e4779`, with only the tests added: 15 of the 19 integration tests and 3 unit tests failed. The 4 that passed are the controls (a link that stays inside, a plain lock, a plain migration). The `util`, `registry`, `generator` and wizard unit tests use functions or signatures this change adds, so they could not run on the base; their write sites are shown red through the integration tests, except the wizard, which is interactive and has no test on the base.

Not driven: a live `resolve --remote`, which needs the network, and the wizard's prompts.

Then the full `fledge lanes run verify` lane locally, and `cargo test` in a local Linux container, since the private fork runs no CI.

## Where these lessons go

- `specs/util/util.3md`
- `specs/hooks/hooks.3md`
- `specs/agents/agents.3md`
- `specs/mcp/mcp.3md`
- `specs/registry/registry.3md`
- `specs/cmd_resolve/cmd_resolve.3md`
- `specs/cmd_migrate/cmd_migrate.3md`
- `specs/generator/generator.3md`
- `specs/cmd_scaffold/cmd_scaffold.3md`
- `specs/cmd_new/cmd_new.3md`
- `specs/cmd_wizard/cmd_wizard.3md`
- `specs/cmd_import/cmd_import.3md`
