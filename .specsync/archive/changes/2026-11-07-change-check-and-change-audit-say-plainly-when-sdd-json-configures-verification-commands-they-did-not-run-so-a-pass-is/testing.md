---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: testing
---

# Testing

Failing first on b3ca5f97 with implementation unchanged (OBS-product-test-693137d491cd153adf290ec2): plain check, committed check, and audit have no stderr note; JSON has no `configured_commands_not_run`; the repository pin finds four top-level commands. After Leif's narrowed 3b decision, the repository pin was rerun and fails only on those four top-level commands while its 13-component and 3-strict retention assertions pass.

After implementation, run all committed probes:

- `change_check_notes_the_configured_commands_it_did_not_run`: three lists in policy order, deduplication, exact stderr prefix, `to stop this note`, no stdout contamination, and no spawned reporter.
- `change_check_commit_notes_the_configured_commands_it_did_not_run`: the committed path prints the note.
- `change_audit_notes_the_configured_commands_it_did_not_run`: text note and JSON list.
- `an_empty_policy_gets_no_note_about_unrun_commands`: no text note and explicit `[]` in check and audit JSON, including empty verification output.
- `this_repository_configures_no_verification_command`: top-level list empty; 13 component modules and 3 strict commands retained.

Run the complete integration suite to locate additive-stderr incompatibilities, then `target/release/specsync check --strict --require-coverage 100 --force`, `fledge lanes run verify`, and `fledge trust verify`. Record affected Chough cases and the full-lane result separately from native spec↔code verification.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-unrun-commands-001 | The five probes above, full integration suite, and no-spawn marker assertion |
