---
kind: session
id: 2026-11-19-1104-terminal-evidence-isolation
aliases: [1104-terminal-evidence-isolation]
objective: One change whose records fail validation no longer decides another change's terminal evidence (#1104)
intent_scope: CHANGE
modules: [change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
pr: ""
criteria: [CHANGE-7, CHANGE-8]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-1104-red-s1-s2-at-418bd2ec
  - 2026-11-19-1104-red-dependency-tests-before-the-fix
  - 2026-11-19-1104-verify-lane-at-b3c2cc0b
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Red first: the shared 'changed after acceptance' constant; then the three audit test tightenings"
---
# One change whose records fail validation no longer decides another change's terminal evidence (#1104)

## Goal

One change whose records fail validation no longer decides another change's terminal evidence (#1104)

## Where it stands

## Decisions and why

- 0xLeif decided "Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence": "approve 1104 as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 15:12Z, on the frozen ss-lead/gates/1104-definition.md, with its three choices as recommended (a bug fix with no new requirement text; an unreadable state.json counts against a successor search only; no warning line in other changes' status); relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence as 0xLeif
- #1104's evidence is recorded against existing criteria, not a new one: CHANGE-7 ('At any moment I can ask what state a change is in and be told the single next thing to do'), since the bug was one change's status reporting another change's broken history, and CHANGE-8 ('I can see every change in flight at once') for change list. Leif approved #1104 as needing no new text, so no criterion is captured; directed by ss-lead (hc t58), 2026-11-19, as #1083 referenced SETUP-2.b
- 0xLeif decided "Leif's relayed approval of #1104's reworded criterion 2 and its declared tests/integration.rs (relayed by hc t4c through ss-lead t58)": "Leif's six rules, rule 6 (a) and (c), for criterion 2; "yes to all 3" for tests/integration.rs" (Criterion 2: a standing OK in Leif's six rules, relayed by hc t4c on 2026-11-19 at about 14:35Z through ss-lead (hc t58): (a) a wording fix that only makes an approved sentence match tested code, and (c) the send-back it needs; applied by ss-lead to this exact text of criterion 2 on 2026-11-19; these are not his words about this text. tests/integration.rs: his one-time yes, "yes to all 3", in a Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 01:00Z, answering item 1, '#1104 and #428 edit four test files their scope doesn't list. May the builder add those paths in the send-backs already planned? No wording or behaviour changes.'; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence as 0xLeif

## What was verified

- [Red first, before the fix: with the tests of 418bd2ec on 880237c8, an archive committed before another change's bad commit (tester2's S1) reads corrupt-history with that other change's 'duplicates state at commit' reason; the broken change's own failure test passes, as it must. Fixed in 71ea5bac.](evidence/2026-11-19-1104-red-s1-s2-at-418bd2ec.md)
- [The dependency tests of 06ac04fe, written after the fix, fail against the pre-fix sources: an accepted change with unchanged inputs and an unreadable state elsewhere does not read exact, and a failed candidate is not named in a successor search.](evidence/2026-11-19-1104-red-dependency-tests-before-the-fix.md)
- [At b3c2cc0b (checked and committed), fledge lanes run verify passed 10/10 steps in 11m42s through one-lane: unit 3099 passed (2 ignored), integration 630, spec check 166/166 files. Includes the S1 integration test through the binary (status, change list, ship-status, audit --archives, a third change shipped). The neutral git config was a prefix on the lane command only (#1092).](evidence/2026-11-19-1104-verify-lane-at-b3c2cc0b.md)

## Open threads

## Timeline

- 2026-11-19T17:17:14Z · start · intent → intent · One change whose records fail validation no longer decides another change's terminal evidence (#1104) ^t-d0a1b6
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T17:22:12Z · advance · intent → design ·  ^t-ded7b1
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T17:23:12Z · skip · design → plan · No visuals or interactions: status verdicts only ^t-70af62
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T17:23:12Z · gate-opened · plan → plan · Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-70b309
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-19T17:23:12Z · gate-closed · plan → plan · Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-67e23d
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - relayed_by:
    - decided_by: 0xLeif
    - words: approve 1104 as written
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 15:12Z, on the frozen ss-lead/gates/1104-definition.md, with its three choices as recommended (a bug fix with no new requirement text; an unreadable state.json counts against a successor search only; no warning line in other changes' status); relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T17:23:19Z · advance · plan → build · native:approved ^t-94f997
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:approved
  - revision: 2cc0897fc00eee7db2b07f5134ba05639af39018
  - by: claude-code (hc t59)
- 2026-11-19T20:14:07Z · advance · build → test · native:verifying ^t-7fcc37
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:verifying
  - revision: b3c2cc0b4c72e55195a205a841bc00afe35d7db4
  - by: claude-code (hc t59)
- 2026-11-19T20:14:15Z · advance · test → accept · native:verifying ^t-68d7c0
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:verifying
  - revision: b3c2cc0b4c72e55195a205a841bc00afe35d7db4
  - by: claude-code (hc t59)
- 2026-11-19T20:14:25Z · gate-opened · accept → accept · Implementation review of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-c7eb62
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: b3c2cc0b4c72e55195a205a841bc00afe35d7db4
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
- 2026-11-20T03:02:26Z · back · accept → build · Rework before review: criterion 2 reworded under standing OK rule 6 (a)+(c); tests/integration.rs declared on Leif's one-time yes; one shared constant for 'changed after acceptance'; three audit test tightenings. History rewritten onto the corrected main, then main a28e3042 merged in. ^t-f25b27
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - gate: native:verifying
  - revision: d29b456bfb9261deb925c93e298608532d3c354b
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 3 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-20T03:02:32Z · gate-closed · build → build · Implementation review of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence ^t-ccb2eb
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: d29b456bfb9261deb925c93e298608532d3c354b
  - by: claude-code (hc t59)
- 2026-11-20T03:07:11Z · gate-opened · build → build · Leif's relayed approval of #1104's reworded criterion 2 and its declared tests/integration.rs (relayed by hc t4c through ss-lead t58) ^t-213bbf
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: f2d135773c8bf5e6b32b41571c16d4aeed7a8ab7
  - holds: build
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-20T03:07:11Z · direct-native-write · build → build · one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state verifying → draft ^t-a2dfe1
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: f2d135773c8bf5e6b32b41571c16d4aeed7a8ab7
  - direct_write:
    - change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: verifying
    - after_state: draft
  - by: claude-code (hc t59)
- 2026-11-20T03:07:11Z · gate-closed · build → build · Leif's relayed approval of #1104's reworded criterion 2 and its declared tests/integration.rs (relayed by hc t4c through ss-lead t58) ^t-968c0e
  - native_change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
  - revision: f2d135773c8bf5e6b32b41571c16d4aeed7a8ab7
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's six rules, rule 6 (a) and (c), for criterion 2; "yes to all 3" for tests/integration.rs
    - where: Criterion 2: a standing OK in Leif's six rules, relayed by hc t4c on 2026-11-19 at about 14:35Z through ss-lead (hc t58): (a) a wording fix that only makes an approved sentence match tested code, and (c) the send-back it needs; applied by ss-lead to this exact text of criterion 2 on 2026-11-19; these are not his words about this text. tests/integration.rs: his one-time yes, "yes to all 3", in a Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 01:00Z, answering item 1, '#1104 and #428 edit four test files their scope doesn't list. May the builder add those paths in the send-backs already planned? No wording or behaviour changes.'; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
    - actor: 0xLeif
  - by: claude-code (hc t59)
