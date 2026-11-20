---
change: never-write-a-project-file-through-a-symbolic-link
artifact: testing
---

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
