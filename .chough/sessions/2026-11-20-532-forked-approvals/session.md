---
kind: session
id: 2026-11-20-532-forked-approvals
aliases: [532-forked-approvals]
objective: A change approved in two clones is not valid until one fresh approval is recorded after the fork (#532)
intent_scope: ""
modules: []
stage: build
stage_status: active
iteration: 1
callers: []
native_change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
pr: ""
criteria: [CHANGE-6, CHANGE-6.a]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-20-532-reproduced-through-the-binary
  - 2026-11-20-532-red-at-5e740a1c
  - 2026-11-20-532-verify-lane-at-b220fecd
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Failing tests first, then the fork check
---
# A change approved in two clones is not valid until one fresh approval is recorded after the fork (#532)

## Goal

A change approved in two clones is not valid until one fresh approval is recorded after the fork (#532)

## Where it stands

## Decisions and why

- 0xLeif decided "Leif's relayed approval of the #532 definition and its four recommended choices (relayed by hc t4c)": ""Approve (Recommended)", to: 'SpecSync #532 (security-labeled): two clones approve the same change, a merge keeps only one side, and check, status and audit all pass while silently dropping the other approval. ... The fix, building on your Nov 13 answers ("build it" and "one fresh approval"): approval validation also looks at both sides of every merge, and if two clones' approvals forked, it refuses until one fresh change approve. Choices, all as recommended: build it after #1120 (#428) merges; already-archived changes aren't re-judged; only definition approvals are covered; a merge that reorders records stays #428's rule. Approve the scope?'" (A Claude Code question dialog in his planning session (hc t4c), 2026-11-20 about 15:06Z, on ss-builder/gates/532-forked-approvals-definition.md, relayed by hc t4c. It builds on his 2026-11-13 answers, "build it" (about 03:49Z) and "One fresh approval (Recommended)" (about 04:04Z), relayed then by coordinator t370); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork as 0xLeif

## What was verified

- [Reproduction (evidence/532-repro/run.sh): two real clones of one origin approve the same change, A merges B with git merge -X ours; on released 6.3.0 and on #428's head 9b75e8ba, change status, change check and change audit --strict exit 0 while clone B's approval is in history and not at HEAD.](evidence/2026-11-20-532-reproduced-through-the-binary.md)
- [Red first at 5e740a1c: the unit fork test and the binary test (two real clones, git merge -X ours) accept the forked history; the guards (approvals that only append, an archived change with a fork in its history) pass. cargo test through one-lane, the neutral git config as a prefix only (#1092).](evidence/2026-11-20-532-red-at-5e740a1c.md)
- [At b220fecd: fledge lanes run verify 10/10 in 13m04s through one-lane (unit 3130, 2 ignored; integration 648; spec check 62/62, 169/169 files). The first lane at 0701ff5c failed #428's a_rewrite_on_a_branch_that_was_merged_in_is_refused_at_the_merge_commit; fixed at 45a4d289 by leaving a divergence that reached first-parent history to #428's rule (approved choice 4). change audit --strict fails only because #428's change is still active on this stacked branch (its evidence reads stale); that clears when #1120 merges and archives it.](evidence/2026-11-20-532-verify-lane-at-b220fecd.md)

## Open threads

## Timeline

- 2026-11-20T15:06:58Z · start · intent → intent · A change approved in two clones is not valid until one fresh approval is recorded after the fork (#532) ^t-70032d
  - revision: 9b75e8baac2c1b254fa77ee8f8a053ed74a12f05
  - by: claude-code (hc t59)
- 2026-11-20T15:11:23Z · gate-opened · intent → intent · Leif's relayed approval of the #532 definition and its four recommended choices (relayed by hc t4c) ^t-d808dd
  - native_change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
  - revision: 9b75e8baac2c1b254fa77ee8f8a053ed74a12f05
  - holds: intent
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-20T15:11:23Z · gate-closed · intent → intent · Leif's relayed approval of the #532 definition and its four recommended choices (relayed by hc t4c) ^t-741167
  - native_change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
  - revision: 9b75e8baac2c1b254fa77ee8f8a053ed74a12f05
  - relayed_by:
    - decided_by: 0xLeif
    - words: "\"Approve (Recommended)\", to: 'SpecSync #532 (security-labeled): two clones approve the same change, a merge keeps only one side, and check, status and audit all pass while silently dropping the other approval. ... The fix, building on your Nov 13 answers (\"build it\" and \"one fresh approval\"): approval validation also looks at both sides of every merge, and if two clones' approvals forked, it refuses until one fresh change approve. Choices, all as recommended: build it after #1120 (#428) merges; already-archived changes aren't re-judged; only definition approvals are covered; a merge that reorders records stays #428's rule. Approve the scope?'"
    - where: A Claude Code question dialog in his planning session (hc t4c), 2026-11-20 about 15:06Z, on ss-builder/gates/532-forked-approvals-definition.md, relayed by hc t4c. It builds on his 2026-11-13 answers, "build it" (about 03:49Z) and "One fresh approval (Recommended)" (about 04:04Z), relayed then by coordinator t370
    - recorded_by: claude-code-t59
    - native: specsync change approve a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-20T15:11:25Z · advance · intent → design · The definition is approved as written and recorded ^t-2ddfd7
  - native_change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
  - revision: 33c0051cc01c2b05618d3711d0e69562a1fc33db
  - by: claude-code (hc t59)
- 2026-11-20T15:11:25Z · advance · design → plan · The definition is approved as written and recorded ^t-d332bf
  - native_change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
  - revision: 33c0051cc01c2b05618d3711d0e69562a1fc33db
  - by: claude-code (hc t59)
- 2026-11-20T15:11:25Z · advance · plan → build · The definition is approved as written and recorded ^t-491998
  - native_change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
  - gate: native:approved
  - revision: 33c0051cc01c2b05618d3711d0e69562a1fc33db
  - by: claude-code (hc t59)
