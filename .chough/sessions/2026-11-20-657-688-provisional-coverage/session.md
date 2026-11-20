---
kind: session
id: 2026-11-20-657-688-provisional-coverage
aliases: [657-688-provisional-coverage]
objective: An in-flight change that declares a shared input covers an accepted change for now, and its archive records which one (#657); each of the two archive refusals over one stale input names the other change and says it is blocked too (#688)
intent_scope: ""
modules: []
stage: design
stage_status: active
iteration: 1
callers: []
native_change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
pr: ""
criteria: [SHIP-13, SHIP-13.a, SHIP-13.b, SHIP-13.c, SHIP-13.d]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-20-657-red-without-the-code-at-e48d22fe
  - 2026-11-20-657-older-binaries-read-coverage-json
  - 2026-11-20-657-verify-lane-at-c7e5daba
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Red first: the provisional-coverage tests with the code set aside; then the code"
---
# An in-flight change that declares a shared input covers an accepted change for now, and its archive records which one (#657); each of the two archive refusals over one stale input names the other change and says it is blocked too (#688)

## Goal

An in-flight change that declares a shared input covers an accepted change for now, and its archive records which one (#657); each of the two archive refusals over one stale input names the other change and says it is blocked too (#688)

## Where it stands

## Decisions and why

- Scope: exactly the frozen ss-lead/gates/657-688-definition.md, which Leif approved as written (relayed by hc t4c through ss-lead t58, 2026-11-19 about 14:17Z). SHIP-13 and .a to .d are the agent's draft he approved with it, captured with that source.
- 0xLeif decided "Leif's definition approval of #657 with #688 and its one cmd_change word (relayed by hc t4c through ss-lead t58)": ""Approve as written" for the definition; "Yes, add it (Recommended)" for REQ-cmd-change-001's one word" (The definition: a Claude Code chat answer to his planning session (hc t4c), 2026-11-19 about 14:17Z, item 8 of items 8 to 10, each shown with its intent sentences marked as the builder's draft, on exactly the frozen ss-lead/gates/657-688-definition.md (SHIP-13 and .a to .d, REQ-change-167, REQ-change-168, four choices). The one word: a question dialog in his planning session (hc t4c), 2026-11-20 about 06:11Z, answering 'OK to add "provisionally-covered" to that list?' for REQ-cmd-change-001's criterion 'exact, successor-covered, or stale'. Both relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one as 0xLeif

## What was verified

- [Red first at e48d22fe, the tests alone (the code set aside): 11 compile errors, in_flight_coverer, TerminalEvidenceValidity::ProvisionallyCovered and ChangeSummary.provisional_coverage do not exist. The ported test also failed at runtime on main a28e3042: the accepted change read stale, not provisionally-covered. cargo test --bin specsync through one-lane, the neutral git config as a prefix only (#1092).](evidence/2026-11-20-657-red-without-the-code-at-e48d22fe.md)
- [The change's fifth criterion: released 6.3.0 (Homebrew) and main a28e3042's 6.4.0 build, on a real archive given a committed coverage.json: change status, --json change status (archived, authenticated-history), change audit --strict, change audit --strict --archives, check --strict, change list and doctor all exit 0. This branch's build reads the same archive with the same exit codes.](evidence/2026-11-20-657-older-binaries-read-coverage-json.md)
- [At c7e5daba (checked and committed), with this tree's release build first on PATH: fledge lanes run verify passed 10/10 in 7m40s through one-lane (unit 3111 passed, 2 ignored, provisional_coverage 11 of 11; integration 644; spec check 62/62, 168/168 files), the neutral git config as a prefix on the lane only (#1092); change audit --strict passes. change audit's exit status is report.errors.is_empty() with or without --strict, so the unit test's no-error-one-warning audit is also --strict's exit 0.](evidence/2026-11-20-657-verify-lane-at-c7e5daba.md)

## Open threads

## Timeline

- 2026-11-20T05:29:40Z · start · intent → intent · An in-flight change that declares a shared input covers an accepted change for now, and its archive records which one (#657); each of the two archive refusals over one stale input names the other change and says it is blocked too (#688) ^t-f2828f
  - revision: a28e30420c12627a384a685cd5d808fc3585b70a
  - by: claude-code (hc t59)
- 2026-11-20T06:45:11Z · gate-opened · intent → intent · Leif's definition approval of #657 with #688 and its one cmd_change word (relayed by hc t4c through ss-lead t58) ^t-b5c6b1
  - revision: a28e30420c12627a384a685cd5d808fc3585b70a
  - holds: intent
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-20T06:45:48Z · gate-closed · intent → intent · Leif's definition approval of #657 with #688 and its one cmd_change word (relayed by hc t4c through ss-lead t58) ^t-99c34d
  - native_change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
  - revision: a28e30420c12627a384a685cd5d808fc3585b70a
  - relayed_by:
    - decided_by: 0xLeif
    - words: "\"Approve as written\" for the definition; \"Yes, add it (Recommended)\" for REQ-cmd-change-001's one word"
    - where: The definition: a Claude Code chat answer to his planning session (hc t4c), 2026-11-19 about 14:17Z, item 8 of items 8 to 10, each shown with its intent sentences marked as the builder's draft, on exactly the frozen ss-lead/gates/657-688-definition.md (SHIP-13 and .a to .d, REQ-change-167, REQ-change-168, four choices). The one word: a question dialog in his planning session (hc t4c), 2026-11-20 about 06:11Z, answering 'OK to add "provisionally-covered" to that list?' for REQ-cmd-change-001's criterion 'exact, successor-covered, or stale'. Both relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-20T07:11:59Z · advance · intent → design ·  ^t-671826
  - native_change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
  - revision: aa368793c0e9aca4a56736852491dde37bdf53ed
  - by: claude-code (hc t59)
