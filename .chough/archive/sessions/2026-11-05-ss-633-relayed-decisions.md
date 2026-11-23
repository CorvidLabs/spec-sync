---
kind: session
id: 2026-11-05-ss-633-relayed-decisions
aliases: [ss-633-relayed-decisions]
objective: Narrow the approval/review promise to recorded, digest-protected claims and record relayed human decisions (#633, audit D7)
intent_scope: ""
modules: []
stage: build
stage_status: active
iteration: 1
callers: []
native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
pr: ""
criteria: []
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "ready for approval to orc t18; after approval: change check --commit, change audit --strict, pre-push, verify, ready for review; then ship, audit, push, CI, merge pinned"
closed: 2026-11-06
outcome: closed-without-done
---
# Narrow the approval/review promise to recorded, digest-protected claims and record relayed human decisions (#633, audit D7)

## Goal

Narrow the approval/review promise to recorded, digest-protected claims and record relayed human decisions (#633, audit D7)

## Where it stands

- Profiled locally (macOS, 20 cores, nextest 0.9.151, main 67aa7dc2): all 3242 tests pass in 811s wall vs ~2000s with cargo test; sys 4364s > user 2398s
- 152 of 3242 tests carry 92% of test time; the slowest (integration eight_workflow_v2_changes_finalize...) takes 343s alone and spawns over 13,000 git processes (929 rev-parse, 186 ls-files, 164 show, 149 rev-list)
- CI rc.2 macos-15 qualify: test build 4m, unit tests 2025s, integration 338s, release build 5m (ubuntu: unit 214s, integration 58s)
- Native definition approval recorded by 0xLeif (via orc t18) at a22d6240
- Repository-fact memo (git_utils) scoped to CLI change commands, project lock and read scopes; invalidated by git writes, verification commands, pre-push gate and gh; live-read guard for race checks
- Batched cat-file --batch reader for show <rev>:<path> blobs and single-path ls-tree; equality tests incl. binary/empty/missing
- Spawn ceilings per command pass: new 13, answer 15, approve 29, check 82, status 50, review 128, finalize 239 (baseline 63/71/89/388/66/323/739); slowest test 5,535 spawns vs 15,563 (35.6%) before the batch reader
- test-nextest task, release-candidate lane, pinned cargo-nextest 0.9.151 install in release.yml (published sha256 verified)
- change check --commit verified (575ddcd9); fledge verify steps pass on pinned 1.89; pre-push green
- Shipped: archived at 2bc2ba92
- HI captured for #633 (b179a4f2): CHANGE-6.b retired, CHANGE-6.d, CHANGE-13..13.d, SHIP-2.e
- Native change record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication drafted: deltas for change (REQ-change-relayed-decision-001, MODIFIED REQ-change-001 and Public API), cmd_change, cli_args; all artifacts complete
- Failing tests first: tests/integration/relayed_decision.rs, 8 discriminators fail on b179a4f2 (unknown --words flag), 2 guards pass
- Sent back to draft as claude (ss-d7) quoting that decision; amended only the recorded_claims row
- Moved Chough records to per-change session ss-633-relayed-decisions; shared .chough/session.json restored to main's copy (orc coordination)
- Codex review on e5716ab2: fixed P1 verdict re-run, P1 correction digests, P2 decider-on-read in bbc46189 with tests; filed #904 for uncommitted review.json recovery
- Merged origin/main twice (af50d85e, efd30230); rebased Public API delta onto #889 rows (29a18a80)
- Second Codex round on 29a18a80: write APIs validate all relay fields; decider check over every review attempt; review-history comparisons ignore relayed_by (tests mutation-checked)
- Merged origin/main 918b6c56 (7df6aef4); check --commit (eeb1a405, 1b21d31f); verify green at 1b21d31f; pushed; CodeQL green, Archive red only for the open change
- Shipped: archived at 1a8d5048 (review pass by 0xLeif via orc, no commit between review and ship); merged origin/main c1987c27+e6b1ce11 into the archive tip (8aeea543, CHANGELOG/INTENT bookkeeping only)
- CI red at 39d3d54d: change audit --strict refused the two paths outside #900's declared scope (Lifecycle gate, Trust); reproduced locally; local verify green at 39d3d54d (2838 unit, 505 integration)
- Merged origin/main 492d316d (#912) at ccc35abd (spec bookkeeping: change.3md planes, change log rows, testing sections); covering change drafted

## Decisions and why

- Agent finding, not a human decision: opt-level 2 test build did not speed the slowest test (386s vs 343s), so an optimized test profile is not the lever; git process spawning is
- Leif (2026-11-05 09:21 MST, Herdr orc tab, relayed by orc t18): approved intent SPEED-2.b.1 and SPEED-2.b.2 and the approach: cut SpecSync git process spawns plus cargo nextest in the qualify lane
- Rejected: optimized test profile as the main lever (opt-level 2 measured 386s vs 343s on the slowest test); larger runner (not needed if spawns drop)
- Leif (2026-11-05 09:35 MST, orc tab): approved the four-part scope; qualify timeout returns to 45 in a one-line follow-up after the next RC's macOS qualify measures well under 45, not in this change
- Leif (2026-11-05 18:23 MST, orc tab, relayed by orc t18): "Record and merge" for PR #869; review recorded as 0xLeif (pass) at 113893d8; acceptance granted
- Leif (relayed by orc t18, 2026-11-05 evening): approved the shared relayed_by shape in ch-d7's proposal section 1: decided_by, words verbatim, where incl. relay chain, recorded_by; all required, non-blank, <=4096 B, unknown keys refused, absent not null; boundary line 'a relayed claim, not authenticated'. Use exactly that shape in SpecSync approval/review records and status, plus the #633 wording narrowing
- decided: SpecSync flags match Chough's (--words, --where, --recorded-by) and decided_by is copied from --actor/--reviewer because one record must not name two deciders
- Leif (2026-11-05 20:19 MST, Herdr orc tab, relayed by orc t18): intent "Approve" as proposed (retire CHANGE-6.b, add CHANGE-6.d, CHANGE-13 + 13.a-d, SHIP-2.e); review digest "Keep it out"; ss-testgaps' CHANGE-13 moves to CHANGE-14
- Leif (2026-11-05 20:35 MST, Herdr orc tab, relayed by orc t18): approved at 6a40e2a2 ('Approve', recorded by orc), then 'Yes, amend and re-approve': send back, amend the recorded_claims Public API row to Result<Vec<RecordedClaim>, String>, re-approve
- Leif (2026-11-05, Herdr orc tab, relayed by orc t18): re-approval recorded by orc as 0xLeif at c7c3ee5e per 'Yes, amend and re-approve'
- Leif (2026-11-05, Herdr orc tab, relayed by orc t18): 'Yes' to re-approval after #889 moved the Public API delta (CHANGE-11); recorded by orc as 0xLeif at 29a18a80
- Leif standing rule (2026-11-05 22:43 MST, relayed by orc t18): a PR he approved that only gets origin/main merged in (no reviewed-content change, checks green) merges at the new head without asking again; reviewed-content changes come back for review
- Leif (2026-11-06, GitHub review relayed by orc t18; his words to the orc: 'im telling you to approve and continue all of these... approve as me'): #900 review recorded by orc as 0xLeif, pass, at 1b21d31f; acceptance granted
- Leif (2026-11-06, relayed by orc t18): 'Keep done fixes, defer rest': new Codex P2s deferred to #926 and #927, uncommitted-review recovery to #904; all 9 #900 threads replied and resolved
- Leif (2026-11-06, orc tab, relayed by orc t18): 'Do the covering change': option A, no-spec-change change covering src/change/collect_requirement_ids.rs and tests/integration/change.rs, acceptance criteria CHANGE-6.d and CHANGE-13.b
- Standing (orc t18, 2026-11-06): run 'specsync change audit --strict' in local checks before every push
- Rejected: reverting either file (undoes the P1 correction-digest fix or fails the key-set test); amending the archived #900 package

## What was verified

## Open threads

## Timeline

- 2026-11-05T16:20:45Z · gate-opened · intent → intent · Intent approval for #861 speed work (criteria drafted, not captured in hi yet) ^t-8684c6
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:21:39Z · gate-closed · intent → intent · Intent approval for #861 speed work (criteria drafted, not captured in hi yet) ^t-aa89ba
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:22:17Z · advance · intent → design ·  ^t-e1a700
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:22:17Z · skip · design → plan · CLI and CI change; no visuals or interactions ^t-c3a6c7
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:33:57Z · gate-opened · plan → plan · Scope approval of cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos (specsync change approve --actor 0xLeif), including whether the qualify timeout returns to 45 in this change or in a follow-up after a measured candidate run ^t-f07b37
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: 67aa7dc25ab8f1ddb732b906b933bb778e2c86aa
- 2026-11-05T16:37:53Z · gate-closed · plan → plan · Scope approval of cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos (specsync change approve --actor 0xLeif), including whether the qualify timeout returns to 45 in this change or in a follow-up after a measured candidate run ^t-bcc14e
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: a22d6240cbce18e4807840bfd89d9ec81a06a339
- 2026-11-05T16:37:54Z · advance · plan → build · native:approved ^t-f9fe8d
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - gate: native:approved
  - revision: a22d6240cbce18e4807840bfd89d9ec81a06a339
- 2026-11-06T03:15:25Z · reset · build → intent · #861 cycle archived and merged (PR #869); starting plan row S8, issue #633 (audit D7) ^t-3f00db
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
  - refresh: intent: HI criteria and FRD; append new criteria, never rewrite approved ones, design: canonical prototype and companion; their human approval covers the old version, plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, native: the definition is archived; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos`; Chough does not change native state.
- 2026-11-06T03:19:10Z · gate-opened · intent → intent · Intent + scope approval for #633 (HI draft: retire CHANGE-6.b, add CHANGE-6.d, CHANGE-13.a-d, SHIP-2.e; review digest choice, recommended: relay excluded) ^t-981fcd
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T03:31:07Z · gate-closed · intent → intent · Intent + scope approval for #633 (HI draft: retire CHANGE-6.b, add CHANGE-6.d, CHANGE-13.a-d, SHIP-2.e; review digest choice, recommended: relay excluded) ^t-7944b9
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: b179a4f254af1ddada0671eece1f8616729e2eff
- 2026-11-06T03:31:07Z · gate-opened · intent → intent · Native scope approval of record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication (orc relays Leif's decision) ^t-16e47e
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: b179a4f254af1ddada0671eece1f8616729e2eff
- 2026-11-06T03:31:12Z · gate-closed · intent → intent · Native scope approval of record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication (orc relays Leif's decision) ^t-349a77
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: b179a4f254af1ddada0671eece1f8616729e2eff
- 2026-11-06T03:31:12Z · advance · intent → design ·  ^t-854635
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: b179a4f254af1ddada0671eece1f8616729e2eff
- 2026-11-06T03:31:12Z · skip · design → plan · CLI and record change; no visuals or interactions ^t-c8e9be
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: b179a4f254af1ddada0671eece1f8616729e2eff
- 2026-11-06T03:31:12Z · gate-opened · plan → plan · Native scope approval of record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication (orc relays Leif's decision) ^t-19748f
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: b179a4f254af1ddada0671eece1f8616729e2eff
- 2026-11-06T03:38:26Z · gate-closed · plan → plan · Native scope approval of record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication (orc relays Leif's decision) ^t-1ddc5e
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - revision: c7c3ee5e1eee7514b5b2ca9d1161c7ccc19be88b
- 2026-11-06T03:38:27Z · advance · plan → build · native:approved ^t-c751c3
  - native_change: cover-the-two-paths-900-changed-outside-its-declared-scope
  - gate: native:approved
  - revision: c7c3ee5e1eee7514b5b2ca9d1161c7ccc19be88b

Closed 2026-11-06: closed-without-done.

## Receipts

```yaml
receipts: []
```
