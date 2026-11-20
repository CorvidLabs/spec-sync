---
kind: session
id: 2026-11-19-1081-vanished-requirements
aliases: [1081-vanished-requirements]
objective: check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58)
intent_scope: ""
modules: [change, cmd_change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
pr: ""
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-covers-req-change-169-and-req-cmd-change-042-the-latter-not
  - 2026-11-19-update-onto-main-2cc0897f-merge-5421ffb3-with-ss-lead-s-five
  - 2026-11-19-history-rewritten-onto-the-corrected-main-m2-7d8370f4-trees
  - 2026-11-20-update-onto-main-10d9bb3f-1100-by-merge-114166fe-six-conflicts
  - 2026-11-20-codex-round-1-on-1108-76b59636-seven-findings-three-p1-four-p2
bugs: []
blockers: []
waiting_on_human:
  - gate: "Leif's review of #1081"
    holds: accept
    drives: review
    opened: 2026-11-20T02:34:09Z
run_consents: []
next: Continue accept
---
# check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58)

## Goal

check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names": "Approve as written" (Claude Code chat with his planning session hc t4c, 2026-11-19 about 14:17Z, on ss-lead's frozen gates/1081-definition.md (CHANGE-17 and .a to .c marked as the agent's draft, REQ-change-169, REQ-cmd-change-042, five choices, no CI-side guard); relayed through ss-lead t58); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names as 0xLeif
- 0xLeif decided "Send-back of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names, run by the builder on Leif's word (origin/HEAD, option A)": "A" (a Claude Code chat message to his planning session hc t4c, 2026-11-20 at about 02:15Z, relayed through ss-lead t58; the question as t4c put it: "A (recommended): only trust origin/HEAD when it actually exists and isn't the branch you're on. Otherwise use origin/main, then origin/master. If none of those works, fall back to the change's starting commit, as your approved wording already says for 'no default branch'. This adds a few words to one approved sentence, through a send-back t4t runs. Side effect: if you work directly on main, the check compares from the change's starting commit instead of origin/main."); recorded by claude-code-t4t; a relayed claim, not authenticated
- 0xLeif decided "Re-approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names (REQ-change-169: origin/HEAD, origin/main or origin/master only when it resolves and is not HEAD's branch)": "A" (a Claude Code chat message to his planning session hc t4c, 2026-11-20 at about 02:15Z, relayed through ss-lead t58; the question as t4c put it: "A (recommended): only trust origin/HEAD when it actually exists and isn't the branch you're on. Otherwise use origin/main, then origin/master. If none of those works, fall back to the change's starting commit, as your approved wording already says for 'no default branch'. This adds a few words to one approved sentence, through a send-back t4t runs. Side effect: if you work directly on main, the check compares from the change's starting commit instead of origin/main." The two insertions are ss-lead's exact texts); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names as 0xLeif

## What was verified

- [Covers REQ-change-169 and REQ-cmd-change-042 (the latter not linked: chough 0.6.0 looks it up under specs/cmd-change). Red first: a_branch_that_lost_requirements_no_removed_delta_names_is_refused (the starling shape) failed on main 3bebe8b0, committed at 5a7b6d68 before the fix. Now 9 unit tests (starling truncation with refused attempt and audit; accounted removals: same change, another active change, an archived change; merged-in vs cherry-picked main removal; move vs renumber with the permanence hint; deleted file and legacy .req.md; base_commit fallback, skip note, unreadable merge-base) and an integration test (ship-status blocker, ship --dry-run, finalize, re-check) pass. Real repos: on CorvidLabs/starling at PR #43's last open commit 3b1dca0 (origin/main at b68d547), released 6.3.0's change check passes (exit 0) and this build refuses (exit 1) naming all 26 lost REQ-StarlingCore ids, the file and the way out; starling main db97480 and Chough main 9d260310 audit clean (exit 0) under both builds; spec-sync's own check on this branch passed. fledge lanes run verify passed with GIT_CONFIG_GLOBAL=/dev/null: 3085 unit and 628 integration tests (including the git-spawn ceiling test), strict specs 62/62, 100% file and LOC coverage; fledge trust verify passed.](evidence/2026-11-19-covers-req-change-169-and-req-cmd-change-042-the-latter-not.md)
- [Update onto main 2cc0897f (merge 5421ffb3) with ss-lead's five review points and the tester's run (t4p passed b8018cc0). Red first: 4347e4aa's tests failed five, each for its point; the depth-1 clone with no origin ref already skipped with the note and is now pinned. Fix b2600d14: the walk and the changed-path list skip .git, .specsync and exclude_dirs (specs_dir '.' plus a change workspace and a vendored copy no longer mask a loss, and the artifact's own id is not read at the reference); an unreadable change accounts for no removal and the lost ids are still named; ids sort naturally; every Git read failure names repair or 'git fetch origin', and 'git fetch --unshallow origin' in a shallow clone missing the merge-base; ship-status exits 1 while the blocker holds (asserted directly, #654 on main). Real repos: starling PR #43's last open commit refused in 1.05 s naming all 26 ids in natural order (6.3.0 passes); Chough #311 and #310 and the mains of Chough 4c764c09, starling db97480 and spec-sync ee76ef1b give the same results as 6.3.0. Timing on spec-sync, the largest: change audit 4.9 s against 6.3.0's 5.0 s; a root-specs walk there would cover 1,901 directories outside .git and .specsync (find: 0.15 s). fledge lanes run verify passed at 967e6bdb (3107 unit, 630 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.](evidence/2026-11-19-update-onto-main-2cc0897f-merge-5421ffb3-with-ss-lead-s-five.md)
- [History rewritten onto the corrected main (M2' 7d8370f4); trees unchanged. It also folds in the two follow-ups made while commits were held: the tester's L2b shape pinned (cherry-pick -x, change created after it), and the merge named as the way out when the default branch removed a lost id too (failing first, then the fix). change check --commit re-recorded verification on the new history (65638bbe, 2d41147d). fledge lanes run verify passed at 2d41147d through one-lane (3108 unit, 630 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.](evidence/2026-11-19-history-rewritten-onto-the-corrected-main-m2-7d8370f4-trees.md)
- [Update onto main 10d9bb3f (#1100) by merge 114166fe. Six conflicts, each two additions at one place, resolved by keeping both sides whole in id order: hi/change.md (CHANGE-15, then CHANGE-17), change.3md (z=55, then z=56), REQ-change-163 then 169, REQ-cmd-change-040 then 042, tests/integration.rs, and INTENT.md's count (59). ss-lead's check_review_carry.py found the branch's 30 reviewed files word for word the same; the INTENT count line is the only difference. change check --commit re-recorded verification (706be086, 24cd747c). Real repos (rule 3) on the merged build: starling PR #43's last open commit 3b1dca0 is refused (exit 1, 0.4 s), naming all 26 lost REQ-StarlingCore ids in natural order (6.3.0 passes). change audit --strict gives the same exit codes and errors as 6.3.0 on spec-sync main 10d9bb3f, Chough main 4c764c09, Chough #311 (75b6bf6e) and #310 (af794540), starling c2863c8 and two other projects with open changes. fledge lanes run verify passed at 24cd747c through one-lane (3114 unit, 645 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.](evidence/2026-11-20-update-onto-main-10d9bb3f-1100-by-merge-114166fe-six-conflicts.md)
- [Codex round 1 on #1108 (76b59636): seven findings (three P1, four P2), all fixed within REQ-change-169's approved text, as ss-lead confirmed. Red first: b8e4d9cd's six new tests failed on 76b59636, each for its finding; the archived-removal test now builds a real archived record. Fix 3d3492ef: (1) a spec registered at the project root brings in the root's own requirements.md and <module>.req.md, not a walk of the tree (a stray file elsewhere is still not read); (2) a finalization retried after acceptance with its workspace active asks the loss question again, and the audit asks it of such an accepted change; (3) the configuration and registry at the reference are read through Git and join the scope; (4, 5) one mechanism gives removal authority: an active change past its draft, or a real archived record that resolves to its package with a valid definition approval; a retired change or a bare archived delta gives none; (6) every Git listing is -z (non-ASCII module path tested with core.quotePath on); (7) the working tree is read to any depth. change check --commit re-recorded verification (903c34af, fbeb3f99). Real repos (rule 3): change audit --strict gives the same exit codes and errors as 6.3.0 and as the build before on spec-sync main 10d9bb3f, Chough main 4c764c09, Chough #311 and #310, starling c2863c8 and two other projects; starling PR #43's last open commit is still refused with all 26 ids. Fixes 1 and 3 changed no real result: none registers a spec at the root, and none moved specs_dir or its registry since the merge-base. fledge lanes run verify passed at fbeb3f99 through one-lane (3120 unit, 645 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.](evidence/2026-11-20-codex-round-1-on-1108-76b59636-seven-findings-three-p1-four-p2.md)

## Open threads

## Timeline

- 2026-11-19T14:23:36Z · start · intent → intent · check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names, naming every lost id (#1081, P1; Leif's 'Approve as written' of the definition, relayed by hc t4c through ss-lead t58) ^t-7d3cb6
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t4t)
- 2026-11-19T14:23:42Z · advance · intent → design ·  ^t-7a0beb
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t4t)
- 2026-11-19T14:25:51Z · skip · design → plan · Record and CLI behaviour only: no visuals or interactions ^t-08d095
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t4t)
- 2026-11-19T14:27:20Z · gate-opened · plan → plan · Scope approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names ^t-2be65a
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T14:27:20Z · gate-closed · plan → plan · Scope approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names ^t-e7e7c5
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve as written
    - where: Claude Code chat with his planning session hc t4c, 2026-11-19 about 14:17Z, on ss-lead's frozen gates/1081-definition.md (CHANGE-17 and .a to .c marked as the agent's draft, REQ-change-169, REQ-cmd-change-042, five choices, no CI-side guard); relayed through ss-lead t58
    - recorded_by: claude-code-t4t
    - native: specsync change approve check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:43Z · direct-native-write · plan → plan · check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names's lifecycle records changed outside Chough: state.json changed, verification.json added, verification-attempts.json added; state approved → verifying ^t-6017c5
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - direct_write:
    - change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - files:
      - state.json: changed
      - verification.json: added
      - verification-attempts.json: added
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:43Z · advance · plan → build · native:verifying ^t-e79c5f
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:47Z · advance · build → test · native:verifying ^t-f87d4c
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:52Z · advance · test → accept · native:verifying ^t-aba92d
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - by: claude-code (hc t4t)
- 2026-11-19T15:09:52Z · gate-opened · accept → accept · Leif's review of #1081 ^t-db1014
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: d8612448bca01f14e9fb6f4c9b58b0aad8428de7
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:19Z · gate-closed · accept → accept · Leif's review of #1081 ^t-1e18cb
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:19Z · gate-opened · accept → accept · Send-back of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names, run by the builder on Leif's word (origin/HEAD, option A) ^t-5cba7a
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - holds: accept
  - drives: none
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:31Z · direct-native-write · accept → accept · check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state stays verifying ^t-34a6ec
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - direct_write:
    - change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:31Z · gate-closed · accept → accept · Send-back of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names, run by the builder on Leif's word (origin/HEAD, option A) ^t-1006e1
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - relayed_by:
    - decided_by: 0xLeif
    - words: A
    - where: a Claude Code chat message to his planning session hc t4c, 2026-11-20 at about 02:15Z, relayed through ss-lead t58; the question as t4c put it: "A (recommended): only trust origin/HEAD when it actually exists and isn't the branch you're on. Otherwise use origin/main, then origin/master. If none of those works, fall back to the change's starting commit, as your approved wording already says for 'no default branch'. This adds a few words to one approved sentence, through a send-back t4t runs. Side effect: if you work directly on main, the check compares from the change's starting commit instead of origin/main."
    - recorded_by: claude-code-t4t
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:47Z · direct-native-write · accept → accept · check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state verifying → draft ^t-837851
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - direct_write:
    - change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: verifying
    - after_state: draft
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:47Z · back · accept → plan · REQ-change-169 gains two insertions on Leif's 'A' (2026-11-20 about 02:15Z): origin/HEAD, origin/main or origin/master counts as the default branch only when it resolves and is not the branch HEAD is on; re-approval follows at plan ^t-bfc5c8
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 5 observation(s) to re-check with `chough status`
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:56Z · gate-opened · plan → plan · Re-approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names (REQ-change-169: origin/HEAD, origin/main or origin/master only when it resolves and is not HEAD's branch) ^t-f508c6
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-20T02:18:56Z · gate-closed · plan → plan · Re-approval of check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names (REQ-change-169: origin/HEAD, origin/main or origin/master only when it resolves and is not HEAD's branch) ^t-7f2d17
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: 9293ee73d08cbf3c76f5233e67d329faded519c4
  - relayed_by:
    - decided_by: 0xLeif
    - words: A
    - where: a Claude Code chat message to his planning session hc t4c, 2026-11-20 at about 02:15Z, relayed through ss-lead t58; the question as t4c put it: "A (recommended): only trust origin/HEAD when it actually exists and isn't the branch you're on. Otherwise use origin/main, then origin/master. If none of those works, fall back to the change's starting commit, as your approved wording already says for 'no default branch'. This adds a few words to one approved sentence, through a send-back t4t runs. Side effect: if you work directly on main, the check compares from the change's starting commit instead of origin/main." The two insertions are ss-lead's exact texts
    - recorded_by: claude-code-t4t
    - native: specsync change approve check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-20T02:33:48Z · direct-native-write · plan → plan · check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state approved → verifying ^t-1f0cbc
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: b6869d9a6a2360949c3400d113d298d84f7386ee
  - direct_write:
    - change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t4t)
- 2026-11-20T02:33:48Z · advance · plan → build · native:verifying ^t-531b57
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: b6869d9a6a2360949c3400d113d298d84f7386ee
  - by: claude-code (hc t4t)
- 2026-11-20T02:33:51Z · advance · build → test · native:verifying ^t-174325
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: b6869d9a6a2360949c3400d113d298d84f7386ee
  - by: claude-code (hc t4t)
- 2026-11-20T02:33:55Z · advance · test → accept · native:verifying ^t-bc0233
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - gate: native:verifying
  - revision: b6869d9a6a2360949c3400d113d298d84f7386ee
  - by: claude-code (hc t4t)
- 2026-11-20T02:34:09Z · gate-opened · accept → accept · Leif's review of #1081 ^t-6f9913
  - native_change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
  - revision: b6869d9a6a2360949c3400d113d298d84f7386ee
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
