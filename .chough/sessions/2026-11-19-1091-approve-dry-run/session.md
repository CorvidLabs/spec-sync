---
kind: session
id: 2026-11-19-1091-approve-dry-run
aliases: [1091-approve-dry-run]
objective: specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)
intent_scope: CHANGE
modules: [change, cmd_change, cli_args]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
pr: ""
criteria: [CHANGE-15, CHANGE-15.a, CHANGE-15.b, CHANGE-15.c]
requirements: []
cases: []
design: []
files:
  - src/change/approval_checks.rs
  - src/change/validate_loaded_change.rs
  - src/change/sdd_version.rs
  - src/commands/change.rs
  - src/cli.rs
  - tests/integration/approve_dry_run.rs
  - src/change_tests/approval_dry_run.rs
observations:
  - 2026-11-19-1091-red-first-tests-fail-before-the-dry-run-exists
  - 2026-11-19-1091-verify-lane-at-c4dba3f6
  - 2026-11-19-1091-red-unreplayable-journal-at-b3573eaa
  - 2026-11-19-1091-verify-lane-at-003a14e6
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
    holds: accept
    drives: review
    opened: 2026-11-19T06:37:41Z
run_consents: []
next: Pushed on feat/1091-approve-dry-run; no PR until ss-lead says. Then Leif's review, relayed by hc t4c through ss-lead; record no review before that.
---
# specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)

## Goal

specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091)

## Where it stands

- Captured CHANGE-15, CHANGE-15.a, CHANGE-15.b and CHANGE-15.c in hi/change.md with their provenance
- Created the draft change with the approved title, specs, paths and interview answers; wrote the three deltas (REQ-change-163 with Leif's exception, REQ-cmd-change-040, REQ-cli-args-027), word for word against the approved text, and every selected artifact
- Failing tests: 1f5c09de
- Split building the approval record from writing it, for the plain and the portable approval (uncommitted, compiles)
- Implemented the dry run (f61bbf0f) after the failing tests (1f5c09de); change check materialized REQ-change-163, REQ-cmd-change-040 and REQ-cli-args-027 with their history fragments; verify lane green at c4dba3f6
- Tester's finding on 37496bdf fixed, red test first: the dry run gives change approve's own refusal for a journal recovery cannot replay; recovery split into its reading and restoring halves; re-checked; lane 10/10 at 003a14e6; site checks pass

## Decisions and why

- 0xLeif decided "Intent approval of #1091 (CHANGE-15 and its three sub-criteria)": "OK, with (a) fixed" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition; relayed by hc t4c through ss-lead (hc t58). CHANGE-15.a is the wording t4c put to him as that fix, which he chose with these words; he did not write it); recorded by claude-code-t59; a relayed claim, not authenticated
- decided: REQ-change-163's first-problem sentence carries the exception 'except when an interrupted lifecycle write is waiting to be recovered' because that is the exception Leif approved with 'OK, with (a) fixed' and t4c confirmed the sentence (relayed through ss-lead hc t58, 2026-11-19)
- 0xLeif decided "Scope approval of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing": "Approve" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition: the requirements, the criteria and the four choices (which checks are skipped, no lock, the interrupted write, the actor as its own check). REQ-change-163's first-problem sentence carries the exception he approved, confirmed by hc t4c. Relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing as 0xLeif
- decided: the second check is named lifecycle_lock, not pending_write as in the working JSON shape, because it reports both things the lock step can refuse on: a symlink in the lock path and an interrupted write (the builder's choice inside the approved shape, which Leif saw marked as working)

## What was verified

- [Red first: on main 3bebe8b0 with the approved #1091 change and only the new tests, all 8 tests of tests/integration/approve_dry_run.rs fail with 'unexpected argument --dry-run found', and cli::tests::change_approve_accepts_a_dry_run fails to parse the flag. The feature does not exist yet; this is the expected failure, not a bug.](evidence/2026-11-19-1091-red-first-tests-fail-before-the-dry-run-exists.md)
- [At c4dba3f6 (#1091 on main 3bebe8b0), fledge lanes run verify passed 10/10 steps in 10m37s: fmt, lint and check-types, cargo test (unit 3083 passed, 2 ignored; integration 635 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. It includes the 8 tests of tests/integration/approve_dry_run.rs (each faulty draft from the Chough tester's list gets one message from the dry run and from the real approve, with not one file changed; a clean draft; two problems in one run; a missing change; a sent-back change; the relay flags; an interrupted write), the 5 unit tests for the in-memory binding and raise and the skip rules, and the flag's parsing test. The lane's processes ran with a neutral global git config (#1092).](evidence/2026-11-19-1091-verify-lane-at-c4dba3f6.md)
- [Red-first, for the tester's finding on 37496bdf: at b3573eaa (the test alone) approve_dry_run::the_dry_run_gives_the_approvals_own_refusal_for_a_journal_it_cannot_replay fails. With a change-transaction.json that does not parse, the dry run says an interrupted write is 'waiting to be recovered', where change approve refuses with 'invalid transaction journal at ...'. Fixed in e24bdbf9; not a new bug.](evidence/2026-11-19-1091-red-unreplayable-journal-at-b3573eaa.md)
- [At 003a14e6 (fix e24bdbf9, change re-checked), fledge lanes run verify passed 10/10 steps in 11m04s: fmt, lint and check-types, cargo test (unit 3083 passed, 2 ignored; integration 636 passed), release build, strict spec check (62 specs) and the CI script tests. It includes the nine approve_dry_run tests; the new one covers a journal that does not parse, one whose integrity envelope does not hold and one that names a path outside the project, each compared word for word with the real approve. The site's tests (48), lint and build pass for the edited cli.md (site-1091b.log). The lane's processes ran with a neutral global git config (#1092)](evidence/2026-11-19-1091-verify-lane-at-003a14e6.md)

## Open threads

## Timeline

- 2026-11-19T05:07:57Z · start · intent → intent · specsync change approve <id> --dry-run runs the approval's own checks, reports the first refusal from each and writes nothing, so Chough's change ready can call it instead of copying SpecSync's rules (#1091) ^t-07b2c9
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T05:08:26Z · gate-opened · intent → intent · Intent approval of #1091 (CHANGE-15 and its three sub-criteria) ^t-b80379
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: intent
  - drives: none
  - by: claude-code (hc t59)
- 2026-11-19T05:08:26Z · gate-closed · intent → intent · Intent approval of #1091 (CHANGE-15 and its three sub-criteria) ^t-37a8d7
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: OK, with (a) fixed
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition; relayed by hc t4c through ss-lead (hc t58). CHANGE-15.a is the wording t4c put to him as that fix, which he chose with these words; he did not write it
    - recorded_by: claude-code-t59
  - by: claude-code (hc t59)
- 2026-11-19T05:08:31Z · advance · intent → design ·  ^t-124ef7
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T05:08:31Z · skip · design → plan · CLI behaviour only: no visuals or interactions ^t-153302
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t59)
- 2026-11-19T05:26:59Z · gate-opened · plan → plan · Scope approval of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing ^t-a6759a
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - revision: a5ec540b461ce962a7d05bb521a807dd10e6dcd8
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-19T05:26:59Z · gate-closed · plan → plan · Scope approval of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing ^t-cf7720
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - revision: a5ec540b461ce962a7d05bb521a807dd10e6dcd8
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:06Z, on the frozen copy of the #1091 definition: the requirements, the criteria and the four choices (which checks are skipped, no lock, the interrupted write, the actor as its own check). REQ-change-163's first-problem sentence carries the exception he approved, confirmed by hc t4c. Relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T05:27:30Z · advance · plan → build · native:approved ^t-afa7b9
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:approved
  - revision: a5ec540b461ce962a7d05bb521a807dd10e6dcd8
  - by: claude-code (hc t59)
- 2026-11-19T06:37:37Z · direct-native-write · build → build · change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state stays verifying ^t-bc7123
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - revision: 22bbf2888974050b186d5d7f49c149a0513573d6
  - direct_write:
    - change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t59)
- 2026-11-19T06:37:37Z · advance · build → test · native:verifying ^t-bb5d77
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:verifying
  - revision: 22bbf2888974050b186d5d7f49c149a0513573d6
  - by: claude-code (hc t59)
- 2026-11-19T06:37:41Z · advance · test → accept · native:verifying ^t-553970
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:verifying
  - revision: 22bbf2888974050b186d5d7f49c149a0513573d6
  - by: claude-code (hc t59)
- 2026-11-19T06:37:41Z · gate-opened · accept → accept · Implementation review of change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing ^t-0b4fd2
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - revision: 22bbf2888974050b186d5d7f49c149a0513573d6
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
- 2026-11-19T07:15:32Z · back · accept → build · tester2's finding on #1091 at 37496bdf: with a transaction journal that cannot be read, the dry run must give the refusal change approve gives ^t-29fb07
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:verifying
  - revision: 37496bdf7423ebcbdc85c80096e4f14f8101801d
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 2 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T07:32:13Z · advance · build → test · native:verifying ^t-390994
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:verifying
  - revision: 003a14e6351565712fc5cc59ddd24c8d11fbe6bc
  - by: claude-code (hc t59)
- 2026-11-19T07:32:14Z · advance · test → accept · native:verifying ^t-cec9ae
  - native_change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
  - gate: native:verifying
  - revision: 003a14e6351565712fc5cc59ddd24c8d11fbe6bc
  - by: claude-code (hc t59)
