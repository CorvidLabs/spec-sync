---
spec: cmd_scaffold.spec.md
---

## Tasks

- [x] Add integration tests for this command's CLI behavior — Evidence: `scaffold_auto_detects_single_source_file` and `scaffold_rejects_module_name_path_traversal`.

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented

## Gaps

- No dedicated test file for this command module

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.

- [x] Implement the approved exact-file creation selection correction with confined no-follow metadata traversal, explicit mapping precedence and causal focused regressions. Full final-source and independent release qualification remain separate.
- [x] Create the spec through `generator::create_spec_file` and exit 1 on a refused registry (REQ-cmd-scaffold-007). Evidence: `tests/integration/project_links.rs` `spec_creation_*`, `scaffold_refuses_a_registry_linked_outside_the_project`.
