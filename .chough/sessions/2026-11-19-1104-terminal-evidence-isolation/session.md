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
  - 2026-11-20-1104-red-shared-constant-at-31b3a26a
  - 2026-11-20-1104-verify-lane-at-880dedcb
  - 2026-11-20-1104-codex-round-1-at-a5befd2b
  - 2026-11-20-1104-codex-round-2-at-2e06e357
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Push and open a normal PR (Archive red on purpose until its merge turn); no review or ship before Leif's relayed review
---
# One change whose records fail validation no longer decides another change's terminal evidence (#1104)

## Goal

One change whose records fail validation no longer decides another change's terminal evidence (#1104)

## Where it stands

- Rework built and checked at 880dedcb: main a28e3042 merged in, criterion 2 reworded and tests/integration.rs declared on the relayed sources, the shared constant red then green, the three audit assertions; lane 10/10 and change audit --strict pass

## Decisions and why

- 0xLeif decided "Approve the scope of one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence": "approve 1104 as written" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 15:12Z, on the frozen ss-lead/gates/1104-definition.md, with its three choices as recommended (a bug fix with no new requirement text; an unreadable state.json counts against a successor search only; no warning line in other changes' status); relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence as 0xLeif
- #1104's evidence is recorded against existing criteria, not a new one: CHANGE-7 ('At any moment I can ask what state a change is in and be told the single next thing to do'), since the bug was one change's status reporting another change's broken history, and CHANGE-8 ('I can see every change in flight at once') for change list. Leif approved #1104 as needing no new text, so no criterion is captured; directed by ss-lead (hc t58), 2026-11-19, as #1083 referenced SETUP-2.b
- 0xLeif decided "Leif's relayed approval of #1104's reworded criterion 2 and its declared tests/integration.rs (relayed by hc t4c through ss-lead t58)": "Leif's six rules, rule 6 (a) and (c), for criterion 2; "yes to all 3" for tests/integration.rs" (Criterion 2: a standing OK in Leif's six rules, relayed by hc t4c on 2026-11-19 at about 14:35Z through ss-lead (hc t58): (a) a wording fix that only makes an approved sentence match tested code, and (c) the send-back it needs; applied by ss-lead to this exact text of criterion 2 on 2026-11-19; these are not his words about this text. tests/integration.rs: his one-time yes, "yes to all 3", in a Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 01:00Z, answering item 1, '#1104 and #428 edit four test files their scope doesn't list. May the builder add those paths in the send-backs already planned? No wording or behaviour changes.'; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence as 0xLeif
- History rewritten onto the corrected main. The commit ids this session's notes and the change's verification records name map to: 880237c8 -> ff30c9cd, 418bd2ec -> 2ce39d47, 71ea5bac -> fd64caa9, 06ac04fe -> 49e4f183, 1d27ec5a -> c16b371f, b433de44 -> c869e8be, 0dbce8cb -> 856ed85c, 39496a1e -> a4c84e4e, b3c2cc0b -> 97bd2a93, 38b2dbfb -> 47df755c, 21233b79 -> d29b456b.
- The change's base_commit 2cc0897f is main before the history was rewritten onto the corrected main; its tree equals 7d8370f4's.

## What was verified

- [Red first, before the fix: with the tests of 418bd2ec on 880237c8, an archive committed before another change's bad commit (tester2's S1) reads corrupt-history with that other change's 'duplicates state at commit' reason; the broken change's own failure test passes, as it must. Fixed in 71ea5bac.](evidence/2026-11-19-1104-red-s1-s2-at-418bd2ec.md)
- [The dependency tests of 06ac04fe, written after the fix, fail against the pre-fix sources: an accepted change with unchanged inputs and an unreadable state elsewhere does not read exact, and a failed candidate is not named in a successor search.](evidence/2026-11-19-1104-red-dependency-tests-before-the-fix.md)
- [At b3c2cc0b (checked and committed), fledge lanes run verify passed 10/10 steps in 11m42s through one-lane: unit 3099 passed (2 ignored), integration 630, spec check 166/166 files. Includes the S1 integration test through the binary (status, change list, ship-status, audit --archives, a third change shipped). The neutral git config was a prefix on the lane command only (#1092).](evidence/2026-11-19-1104-verify-lane-at-b3c2cc0b.md)
- [Red first at 31b3a26a: the_successor_search_reads_the_words_both_writers_write fails to compile, E0425 cannot find value CHANGED_AFTER_ACCEPTANCE, before the shared constant exists; everything else in the test compiles. cargo test --bin specsync through one-lane, the neutral git config as a prefix only (#1092).](evidence/2026-11-20-1104-red-shared-constant-at-31b3a26a.md)
- [At 880dedcb (main a28e3042 merged in, sent back, reworded criterion 2 approved, checked and committed), with this tree's own release build (specsync 6.4.0) first on PATH: fledge lanes run verify passed 10/10 steps in 9m37s through one-lane (unit 3106 passed, 2 ignored, including #1104's six unit tests and the shared-constant test; integration 645, including the binary test's three audit assertions: plain change audit exits 1 with the error on a change open and archived at once and passes on S1, change audit --archives exits 1 naming it on S1; spec check 62/62 specs, 168/168 files), the neutral git config as a prefix on the lane only (#1092). specsync change audit --strict against origin/main a28e3042 passes.](evidence/2026-11-20-1104-verify-lane-at-880dedcb.md)
- [Codex's and corvid-agent's first round on #1114: red first at f39d6921 (5 of 11 fail: deadline, collision verdict, a no-spec change named, rolled-back retirement, over-blame) and ca1f054b (parity test, E0425); fixed at ae4172a4; checked at a5befd2b. With this tree's release build first on PATH, fledge lanes run verify passed 10/10 in 9m00s through one-lane (unit 3112 passed, 2 ignored; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes.](evidence/2026-11-20-1104-codex-round-1-at-a5befd2b.md)
- [Codex's second round on #1114: red first at 9ea421e6 (3 of 3 fail: a healthy legacy accepted change told to reopen, a failed archive's verdict recomputed in the scope, a rollback to an open workspace with no archive not reported); fixed and checked at 2e06e357. With this tree's release build first on PATH, fledge lanes run verify passed 10/10 in 14m35s through one-lane (unit 3115 passed, 2 ignored; integration 645; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes.](evidence/2026-11-20-1104-codex-round-2-at-2e06e357.md)

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
