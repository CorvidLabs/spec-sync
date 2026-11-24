---
spec: cmd_wizard.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add integration tests for this command's CLI behavior

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Write the wizard's spec through `generator::create_spec_file`, in a step that can be tested without a terminal (REQ-cmd-wizard-003). Evidence: `wizard_write_refuses_a_dangling_spec_link_and_a_linked_spec_directory`.
- [x] Walk or list configured folders only through the confined walk, and keep the read cache out of the Git directory (REQ-cmd-wizard-004).

## Gaps

- No dedicated test file for this command module

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
