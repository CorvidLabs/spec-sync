---
kind: session
id: 2026-11-06-ss-904-relay-repair
aliases: [ss-904-relay-repair]
objective: "spec-sync#904: a damaged relayed_by refusal names one recovery that works whether or not the file is committed (specsync change repair-relays), never removing an approval or review attempt"
intent_scope: CHANGE
modules: [change]
stage: plan
stage_status: archived
iteration: 1
native_change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
criteria: [CHANGE-13.e, CHANGE-13.f, CHANGE-13.g]
requirements:
  - REQ-change-relay-repair-001
  - REQ-cmd-change-relay-repair-001
  - REQ-cli-args-relay-repair-001
closed: 2026-11-13
outcome: done
---
# spec-sync#904: a damaged relayed_by refusal names one recovery that works whether or not the file is committed (specsync change repair-relays), never removing an approval or review attempt

## Goal

spec-sync#904: a damaged relayed_by refusal names one recovery that works whether or not the file is committed (specsync change repair-relays), never removing an approval or review attempt

## Where it stands

- Stacked worktree ss-904 on #900 head 83db636b; proposal sent to orc t18; named IDs REQ-change-/REQ-cmd-change-/REQ-cli-args-relay-repair-001 registered by orc
- Failing first: 6 #904 discriminators in tests/integration/relayed_decision.rs fail on 83db636b (5 because the refusal names git checkout HEAD, not repair-relays; 1 because the repair-relays subcommand does not exist); the archived-package guard and the 10 #900 tests pass
- Failing first extended: 8 #904 discriminators fail on 83db636b (5 name git checkout HEAD instead of repair-relays; 3 hit the missing repair-relays subcommand: nothing damaged, archived refusal, not-JSON refusal); the archived guard and the 10 #900 tests pass. Implementation drafted outside the tree (scratchpad draft-904/relay_repair.rs)
- HI captured: CHANGE-13.e, 13.f, 13.g in hi/change.md with attribution
- Native change repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed drafted: ADDED REQ-change-/REQ-cmd-change-/REQ-cli-args-relay-repair-001; MODIFIED REQ-change-relayed-decision-001 (read-refusal bullet only) and change Public API (RelayRepair, RelayRepairReport, repair_relayed_decisions rows; recorded_claims recovery wording); all artifacts complete; merged #900 head 1b21d31f (incl. main 918b6c56)

## Decisions and why

- Leif decided "Leif: approve the #904 intent (CHANGE-13.e/f/g as drafted) and design (specsync change repair-relays), and decide the no-copy case (remove the damaged relayed_by, recommended, or refuse). Proposal logged by orc t18 for Leif's morning.": "Ok yes I approve all" (Herdr orc tab, 2026-11-06 between 06:10 and 06:19 MST, in reply to orc t18's morning brief; relayed by orc t18 to ss-d7 (hc t70)); recorded by claude (ss-d7); a relayed claim, not authenticated
- Leif (2026-11-06 06:1x MST, Herdr orc tab, relayed by orc t18): 'Ok yes I approve all': CHANGE-13.e/f/g as drafted, the new specsync change repair-relays command, and with no surviving copy remove the damaged relayed_by
- Rejected: refusing when no copy of the original survives (leaves the change unreadable; the only way out would erase review files or hand-edit)

## What was verified

## Bugs

None.

## Open threads

## Timeline

- 2026-11-06T07:57:40Z · gate-opened · intent → intent · Leif: approve the #904 intent (CHANGE-13.e/f/g as drafted) and design (specsync change repair-relays), and decide the no-copy case (remove the damaged relayed_by, recommended, or refuse). Proposal logged by orc t18 for Leif's morning. ^t-484159
  - native_change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
  - revision: 83db636b911032a864a0b978bad22535ebdc931d
- 2026-11-06T13:17:48Z · gate-closed · intent → intent · Leif: approve the #904 intent (CHANGE-13.e/f/g as drafted) and design (specsync change repair-relays), and decide the no-copy case (remove the damaged relayed_by, recommended, or refuse). Proposal logged by orc t18 for Leif's morning. ^t-9af667
  - native_change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
  - revision: 61e5a2b026630535f7b563a465cc63f2f69931a3
  - relayed_by:
    - decided_by: Leif
    - words: Ok yes I approve all
    - where: Herdr orc tab, 2026-11-06 between 06:10 and 06:19 MST, in reply to orc t18's morning brief; relayed by orc t18 to ss-d7 (hc t70)
    - recorded_by: claude (ss-d7)
- 2026-11-06T13:17:53Z · advance · intent → design ·  ^t-06cfe7
  - native_change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
  - revision: 61e5a2b026630535f7b563a465cc63f2f69931a3
- 2026-11-06T13:17:58Z · skip · design → plan · CLI-only change: a new specsync subcommand and refusal wording; no visual or interactive design ^t-6b8731
  - native_change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
  - revision: 61e5a2b026630535f7b563a465cc63f2f69931a3
- 2026-11-06T13:25:55Z · pause · plan → plan · Orc t18: park #904 until #900 is done (open-PR work first) ^t-75b609
  - native_change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
  - revision: 5f069712ad798995a09f43bd13f66c7f948e291c

Closed 2026-11-13: done.

## Receipts

```yaml
receipts: []
```
