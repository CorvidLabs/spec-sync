---
spec: cmd_scaffold.spec.md
---

## Tasks

- [x] Add integration tests for this command's CLI behavior — Evidence: `scaffold_auto_detects_single_source_file` and `scaffold_rejects_module_name_path_traversal`.

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] #421: replace add-spec's YAML-null template with the shared renderer
- [x] #421: verify detected exports populate add-spec and scaffold Public API tables

## Gaps

- Inline and integration regressions cover add-spec source filtering/API rows, empty-draft validity, and scaffold export population.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
