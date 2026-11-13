---
kind: evidence
id: 2026-11-07-failing-first-on-main-b3ca5f97-implementation-unchanged-change
aliases: [OBS-product-test-693137d491cd153adf290ec2]
claim: "Failing first on main b3ca5f97 (implementation unchanged): change_check_notes_the_configured_commands_it_did_not_run, change_check_commit_notes_the_configured_commands_it_did_not_run and change_audit_notes_the_configured_commands_it_did_not_run find no note on stderr; an_empty_policy_gets_no_note_about_unrun_commands finds no configured_commands_not_run list in JSON; this_repository_configures_no_verification_command finds the 4 verification_commands in .specsync/sdd.json. Pins proposed REQ-cmd-change-unrun-commands-001 (not yet in a native change)."
result: failed
evidence_kind: product-test
method: automated
revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
content: 785db9ce9980bcac9f53094b96e39e4da490c27994b823db798d801cce82b82a
environment: local
criteria: [SHIP-1.c.3]
requirements: [REQ-change-023]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-07T02:53:55Z
---
**Failing first on main b3ca5f97 (implementation unchanged): change_check_notes_the_configured_commands_it_did_not_run, change_check_commit_notes_the_configured_commands_it_did_not_run and change_audit_notes_the_configured_commands_it_did_not_run find no note on stderr; an_empty_policy_gets_no_note_about_unrun_commands finds no configured_commands_not_run list in JSON; this_repository_configures_no_verification_command finds the 4 verification_commands in .specsync/sdd.json. Pins proposed REQ-cmd-change-unrun-commands-001 (not yet in a native change).**

Recorded as failed in local at revision b3ca5f97aa9c, observed 2026-11-07T02:53:55Z.
