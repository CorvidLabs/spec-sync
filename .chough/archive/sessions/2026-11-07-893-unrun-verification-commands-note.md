---
kind: session
id: 2026-11-07-893-unrun-verification-commands-note
aliases: [893-unrun-verification-commands-note]
objective: change check and change audit say plainly when sdd.json configures verification commands they did not run, so a pass is not read as the project's tests passing (#893)
intent_scope: SHIP
modules: [change, cmd_change]
stage: build
stage_status: archived
iteration: 1
native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
criteria: [SHIP-1.c.3]
requirements: [REQ-change-023, REQ-cmd-change-unrun-commands-001]
observations: [OBS-product-test-693137d491cd153adf290ec2]
closed: 2026-11-13
outcome: done
---
# change check and change audit say plainly when sdd.json configures verification commands they did not run, so a pass is not read as the project's tests passing (#893)

## Goal

change check and change audit say plainly when sdd.json configures verification commands they did not run, so a pass is not read as the project's tests passing (#893)

## Where it stands

- Narrowed this_repository_configures_no_verification_command to empty only verification_commands and pin the retained 13 component and 3 strict commands
- Created and completed the native cmd_change change for #893; status names scope approval as the next action
- Merged origin/main 4936b81b at 2fe73af8; 5 conflicts resolved keeping both histories; focused probes 5/5 green on the merge
- Materialized REQ-cmd-change-unrun-commands-001 in requirements.md; spec v76 contract item 13; strict check 62/62, score 100
- Native change check --commit verified and committed at c194dd7e; fledge trust verify passed (progressive); strict audit passed; pre-push gate PASS; pushed; draft PR #1016 opened

## Decisions and why

- Leif decided "Intent decision for #893 (SHIP-1.c.3)": "1A 2yes 3yes 3b-yes" (Herdr orc tab, relayed by orc t18 to t15w, 2026-11-06 ~19:35 MST); recorded by claude (ss-triage t15w); a relayed claim, not authenticated
- Leif 3yes: stderr note in text on change check and change audit, JSON list configured_commands_not_run, never fails or runs anything; 3b-yes: empty spec-sync's own verification_commands (relayed by orc t18 2026-11-06 ~19:35 MST)
- Leif narrowed 3b: empty only verification_commands and keep the 13 component and 3 strict commands, so the note stays visible for this repository; relayed by orc t18 on 2026-11-06
- Leif decided "Implementation review of #893 (PR #1016) at exact head c194dd7efff9fc7f108756718abf8de501b4b03f, recorded by orc t19f": "approve" (Direct Leif reply to orc t19f ONE-item #893 implementation acceptance / draft PR1016 on 2026-11-07; native scoped PASS recorded as 0xLeif at exact cbfb99d0c68914ac12bf99c4f6417b9a2df6e602 by orc t19f; relayed by orc t19f); recorded by k-ss893 (kimi-code/k3); a relayed claim, not authenticated

## What was verified

- Failing first on main b3ca5f97 (implementation unchanged): change_check_notes_the_configured_commands_it_did_not_run, change_check_commit_notes_the_configured_commands_it_did_not_run and change_audit_notes_the_configured_commands_it_did_not_run find no note on stderr; an_empty_policy_gets_no_note_about_unrun_commands finds no configured_commands_not_run list in JSON; this_repository_configures_no_verification_command finds the 4 verification_commands in .specsync/sdd.json. Pins proposed REQ-cmd-change-unrun-commands-001 (not yet in a native change). ^2026-11-07-failing-first-on-main-b3ca5f97-implementation-unchanged-change

## Bugs

None.

## Open threads

## Timeline

- 2026-11-07T02:46:07Z · gate-opened · intent → intent · Intent decision for #893 (SHIP-1.c.3) ^t-8a36e2
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
- 2026-11-07T02:46:07Z · gate-closed · intent → intent · Intent decision for #893 (SHIP-1.c.3) ^t-563389
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
  - relayed_by:
    - decided_by: Leif
    - words: 1A 2yes 3yes 3b-yes
    - where: Herdr orc tab, relayed by orc t18 to t15w, 2026-11-06 ~19:35 MST
    - recorded_by: claude (ss-triage t15w)
- 2026-11-07T02:46:07Z · advance · intent → design ·  ^t-79b042
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
- 2026-11-07T02:46:08Z · skip · design → plan · CLI text and JSON output only; no visuals or interactions ^t-34f05f
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
- 2026-11-07T03:24:07Z · gate-opened · plan → plan · Scope approval of #893 change change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is, sent to orc t18 ^t-6fa505
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: 22a2dcaca598949dfb3a34907ea2fb0cfd18a820
- 2026-11-07T03:36:48Z · gate-closed · plan → plan · Scope approval of #893 change change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is, sent to orc t18 ^t-55a43f
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: ff2b697781c7c7554083bdd5f70f52b67811653b
- 2026-11-07T03:36:56Z · advance · plan → build · native:approved ^t-d775a1
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - gate: native:approved
  - revision: ff2b697781c7c7554083bdd5f70f52b67811653b
- 2026-11-07T21:24:25Z · gate-opened · build → build · Implementation review of #893 (PR #1016) at exact head c194dd7efff9fc7f108756718abf8de501b4b03f, recorded by orc t19f ^t-8c2f12
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: c194dd7efff9fc7f108756718abf8de501b4b03f
- 2026-11-07T22:05:38Z · gate-closed · build → build · Implementation review of #893 (PR #1016) at exact head c194dd7efff9fc7f108756718abf8de501b4b03f, recorded by orc t19f ^t-a2f91f
  - native_change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
  - revision: 03e05f422c3fe137af046c9665e2606a774e52fe
  - relayed_by:
    - decided_by: Leif
    - words: approve
    - where: Direct Leif reply to orc t19f ONE-item #893 implementation acceptance / draft PR1016 on 2026-11-07; native scoped PASS recorded as 0xLeif at exact cbfb99d0c68914ac12bf99c4f6417b9a2df6e602 by orc t19f; relayed by orc t19f
    - recorded_by: k-ss893 (kimi-code/k3)

Closed 2026-11-13: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-07-failing-first-on-main-b3ca5f97-implementation-unchanged-change
    aliases: OBS-product-test-693137d491cd153adf290ec2
    claim: "Failing first on main b3ca5f97 (implementation unchanged): change_check_notes_the_configured_commands_it_did_not_run, change_check_commit_notes_the_configured_commands_it_did_not_run and change_audit_notes_the_configured_commands_it_did_not_run find no note on stderr; an_empty_policy_gets_no_note_about_unrun_commands finds no configured_commands_not_run list in JSON; this_repository_configures_no_verification_command finds the 4 verification_commands in .specsync/sdd.json. Pins proposed REQ-cmd-change-unrun-commands-001 (not yet in a native change)."
    result: failed
    revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
    environment: local
    content: 785db9ce9980bcac9f53094b96e39e4da490c27994b823db798d801cce82b82a
    observed: 2026-11-07T02:53:55Z
```
