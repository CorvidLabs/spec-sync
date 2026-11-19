---
kind: session
id: 2026-11-13-1038-ship-status-archived-review
aliases: [1038-ship-status-archived-review]
objective: ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)
intent_scope: ""
modules: [change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
pr: ""
criteria: [SHIP-8]
requirements: []
cases: []
design: []
files: [src/change/consider_accepted_evidence_anchor.rs, src/commands/change.rs]
observations:
  - 2026-11-13-at-f52c39ff3-fix-1038-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-at-f52c39ff3-verify-step-4-spec-check-strict-failed-archived
  - 2026-11-13-at-e1919d06f-after-the-public-api-row-spec-check-strict-62
  - 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests-an
  - 2026-11-19-1038-red-control-on-main-3bebe8b0
  - 2026-11-19-1038-verify-lane-at-4d439b39
  - 2026-11-19-1038-red-control-for-the-review-fixes-at-7052e227
  - 2026-11-19-1038-verify-lane-at-612d255d
  - 2026-11-19-1038-red-control-for-the-archive-side-at-d14fcad3
  - 2026-11-19-1038-verify-lane-at-5f5cdf8d
  - 2026-11-19-1038-the-real-archive-reopen-and-re-ship-case
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
    holds: accept
    drives: review
    opened: 2026-11-13T20:07:47Z
run_consents: []
next: Continue build
---
# ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)

## Goal

ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control
- Refreshed onto main 3bebe8b0 on branch fix/ship-status-654-1038-937 with #654 and #937: merged the 11-13 branch with no conflict and re-ran change check; no delta, so no history fragment; red control on main and the verify lane recorded
- Fixed the two review findings on PR #1093 (failing tests 7052e227, fix c697bbde): the own-package tip exemption now needs a deletion and an absent package directory, as CI's classifier does; only a workflow-v2 archive with a valid finalization record reads done by finalize, and any other archived change gets a done review stage that names no review; added the CHANGELOG Fixed line
- Second parity finding on PR #1093 fixed (failing tests d14fcad3, fix e54bc636): a change's own package paths are archive work only in the shape CI's script accepts by status; main's rule for tips with no package path is left as it was; the positive test now builds the real tip of a change archived after a reopen
- Ran the send-back on Leif's relayed word (gate above), exactly as written in ss-lead/gates/1093-send-back-command.md: specsync change send-back ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only --actor 0xLeif --reason "The first acceptance criterion gains 'that finalize archived': a workflow-v1 archive is not done by finalize (review of PR #1093)". It exited 0 and returned the change to draft with its approval cancelled

## Decisions and why

- decided: authorization is Leif's own decision ('Bug fix (Recommended)', option text 'For an archived change, report the review as done by finalize and suggest nothing.', coordinator chat 2026-11-13T04:39Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it
- decided: the approved definition is unchanged and the 11-13 approval stands because Leif said 'Yes, they stand' (relayed by his planning session hc t4c through ss-lead hc t58, 2026-11-18)
- decided: one review answer from Leif on the batch PR counts as the scoped review of each change in it because he said 'Yes, one answer per batch' (relayed by hc t4c through ss-lead hc t58, 2026-11-18); nothing is recorded until it comes
- decided: #1038's first acceptance criterion gains 'that finalize archived' and a clause for an archived change that finalize did not archive, because Leif said 'Approve' to that line (Claude Code chat with his planning session hc t4c, 2026-11-19 about 05:21Z, relayed through ss-lead hc t58); it is not recorded natively yet, because the criterion changes only through change send-back, which Leif runs himself
- 0xLeif decided "Send-back of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only, run by the builder on Leif's word": "The builder runs it, on my word" (Claude Code chat answer to his planning session (hc t4c), 2026-11-19 about 14:05Z, on the send-back command in ss-lead/gates/1093-send-back-command.md; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated
- 0xLeif decided "Approve the scope of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only": "Approve" (Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:21Z, on the new acceptance criterion line for #1038's change in ss-lead/gates/1093-1038-criterion.md ('New'); relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only as 0xLeif
- decided: the 11-13 approval of this change no longer stands: the send-back of 2026-11-19 (run by the builder on Leif's relayed word) cancelled it, the acceptance criterion is now the 'New' line of ss-lead/gates/1093-1038-criterion.md word for word, and Leif's relayed 'Approve' of that line (2026-11-19 about 05:21Z) is the current approval

## What was verified

- [At f52c39ff3 (fix/1038 with origin/main bb7682a78 merged in), fledge lanes run verify steps 1 to 3 passed: fmt, lint and check-types, cargo test (unit 3072 passed, 2 ignored; integration 619 passed), including the commands::change tests for an archived change's review stage and its archive-only tip, and the release build.](evidence/2026-11-13-at-f52c39ff3-fix-1038-with-origin-main-bb7682a78-merged-in.md)
- [At f52c39ff3, verify step 4 spec-check --strict failed: archived_review_is_settled_by_finalize, the crate-visible helper this change adds, had no Public API row in specs/change/change.spec.md. Fixed in e1919d06f in this session by adding the row; found and fixed before review, so no bug record.](evidence/2026-11-13-at-f52c39ff3-verify-step-4-spec-check-strict-failed-archived.md)
- [At e1919d06f, after the Public API row: spec-check --strict (62 specs, 100% file and LOC coverage), release-candidate-test, ci-gate-test, linux-baseline-test, ci-trigger-paths-test, ci-attest-and-summary-test and clippy-all-targets-test passed. Rust sources are unchanged since steps 1 to 3.](evidence/2026-11-13-at-e1919d06f-after-the-public-api-row-spec-check-strict-62.md)
- [Red-first control: on main 18385ef5e with only the new tests, an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review and an_archive_tip_that_removes_its_own_package_files_is_archive_only fail. These are the defects #1038 reports, fixed by this change; not new bugs.](evidence/2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests-an.md)
- [Red-first control: on main 3bebe8b0 with only the new tests, an_archive_tip_that_removes_its_own_package_files_is_archive_only fails (tip class product, not archive_only) and an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review fails (review_currency stale, and the stage says to re-record the review). These are the defects #1038 reports, fixed by this change; not new bugs.](evidence/2026-11-19-1038-red-control-on-main-3bebe8b0.md)
- [At 4d439b39 (#654, #1038 and #937 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 16m07s: fmt, lint and check-types, cargo test (unit 3079 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests, including commands::change::tests::an_archived_changes_review_stage_is_done_by_finalize_and_suggests_no_review and an_archive_tip_that_removes_its_own_package_files_is_archive_only. The lane's processes ran with a neutral global git config: with the machine's own config, which signs tags, four changelog unit tests wait on an editor for a tag message.](evidence/2026-11-19-1038-verify-lane-at-4d439b39.md)
- [Red-first control for the review of PR #1093, at 7052e227 with only the new tests: an_archive_tip_whose_package_is_still_present_is_product fails (a tip that modifies the change's package, still present, reads archive_only where CI picks the product lane), and a_workflow_v1_archive_is_not_done_by_finalize_and_suggests_no_review fails (a workflow-v1 archive holding a review.json reads review_currency current). These are the two review findings, fixed in c697bbde; not new bugs.](evidence/2026-11-19-1038-red-control-for-the-review-fixes-at-7052e227.md)
- [At 612d255d (PR #1093 after the review fixes c697bbde, each change re-checked there), fledge lanes run verify passed 10/10 steps in 29m34s on a busy machine: fmt, lint and check-types, cargo test (unit 3081 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes the four #1038 tests: the archived review stage done by finalize with no blocker, the workflow-v1 archive that is not done by finalize and is told no review, the archive-only tip, and the tip whose package is still present reading product.](evidence/2026-11-19-1038-verify-lane-at-612d255d.md)
- [Red-first control for the second parity finding on PR #1093, at d14fcad3 with only the new tests: deleted_package_files_beside_a_modified_archive_file_are_product fails (the tip reads archive_only), and the_own_package_exemption_agrees_with_the_ci_classifier_script fails on that same tip (the script says archive_only=false). Fixed in e54bc636; not a new bug.](evidence/2026-11-19-1038-red-control-for-the-archive-side-at-d14fcad3.md)
- [At 5f5cdf8d (PR #1093 after the archive-side fix e54bc636, each change re-checked there), fledge lanes run verify passed 10/10 steps in 9m42s: fmt, lint and check-types, cargo test (unit 3084 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes the #1038 tests: the archive tip of a reopened change on its real shape, deleted package files beside a modified archive file, a package still present, the status table, the agreement with CI's classifier script (run with bash and jq), and the two archived review-stage tests.](evidence/2026-11-19-1038-verify-lane-at-5f5cdf8d.md)
- [The real case of #1038, driven through the release binary built at e54bc636 on a throwaway project: a change approved, checked, reviewed and shipped; its delivered file edited; change reopen; check, review and ship again. The re-archive tip is 14 renames into the archive, 2 additions there (finalization.json, verification.json) and 2 deletions from the package, with no modified archive file. CI's classify-ci-paths.sh in name-status mode says archive_only=true, and ship-status says tip_class archive_only, review_currency current, no blockers. The first archive tip agrees too.](evidence/2026-11-19-1038-the-real-archive-reopen-and-re-ship-case.md)

## Open threads

## Timeline

- 2026-11-13T19:23:28Z · start · intent → intent · ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only (#1038) ^t-c7c8f5
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:30Z · advance · intent → design ·  ^t-a0a1bc
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:30Z · skip · design → plan · CLI, CI and docs behaviour only: no visuals or interactions ^t-a59442
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:32Z · advance · plan → build · native:verifying ^t-b0b998
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:32Z · advance · build → test · native:verifying ^t-ac5b84
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: f52c39ff32403bd7e85932e132dd565dfecb2c24
  - by: claude-code (hc t38p)
- 2026-11-13T20:07:47Z · advance · test → accept · native:verifying ^t-5b933b
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: e1919d06f5ce03090fe954ecc8eb99d3064212b6
  - by: claude-code (hc t38p)
- 2026-11-13T20:07:47Z · gate-opened · accept → accept · Implementation review of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only ^t-75dae6
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: e1919d06f5ce03090fe954ecc8eb99d3064212b6
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
- 2026-11-19T05:16:07Z · back · accept → build · Review findings on PR #1093 (Codex P2 twice, corvid-agent changes requested): the archive-only tip class must match CI's classifier, and a workflow-v1 archive must not read done by finalize ^t-7e4077
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: 8313f3ff7f17d50fbd13e1c9296a6e144a6ba572
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 6 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T05:56:33Z · advance · build → test · native:verifying ^t-063ea3
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: 612d255dc6c463fdabdb7fe329f74dcb78de5824
  - by: claude-code (hc t59)
- 2026-11-19T05:56:36Z · advance · test → accept · native:verifying ^t-959d30
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: 612d255dc6c463fdabdb7fe329f74dcb78de5824
  - by: claude-code (hc t59)
- 2026-11-19T06:03:58Z · back · accept → build · Second parity finding on PR #1093 (ss-lead, run against CI's script): the tip rule must match the script by status on the archive side too ^t-8dd417
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: d4f6bee6c60c919e9c1ee805e8eb4d2ec03de5e3
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 8 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T06:20:41Z · advance · build → test · native:verifying ^t-3d4d32
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: 5f5cdf8d41eb8ca4e8f3c3b229e8c5f24f10c5c7
  - by: claude-code (hc t59)
- 2026-11-19T06:20:44Z · advance · test → accept · native:verifying ^t-e15c02
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:verifying
  - revision: 5f5cdf8d41eb8ca4e8f3c3b229e8c5f24f10c5c7
  - by: claude-code (hc t59)
- 2026-11-19T14:07:21Z · gate-opened · accept → accept · Send-back of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only, run by the builder on Leif's word ^t-9bfcab
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - holds: accept
  - drives: none
  - by: claude-code (hc t59)
- 2026-11-19T14:07:21Z · gate-closed · accept → accept · Send-back of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only, run by the builder on Leif's word ^t-c442be
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - relayed_by:
    - decided_by: 0xLeif
    - words: The builder runs it, on my word
    - where: Claude Code chat answer to his planning session (hc t4c), 2026-11-19 about 14:05Z, on the send-back command in ss-lead/gates/1093-send-back-command.md; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
  - by: claude-code (hc t59)
- 2026-11-19T14:07:38Z · direct-native-write · accept → accept · ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state verifying → draft ^t-977136
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - direct_write:
    - change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: verifying
    - after_state: draft
  - by: claude-code (hc t59)
- 2026-11-19T14:07:38Z · back · accept → plan · The send-back returned the change to draft; its criterion changes and Leif's re-approval is recorded at plan ^t-df028d
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 11 observation(s) to re-check with `chough status`
  - by: claude-code (hc t59)
- 2026-11-19T14:08:25Z · gate-opened · plan → plan · Approve the scope of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only ^t-519a30
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-19T14:08:25Z · gate-closed · plan → plan · Approve the scope of ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only ^t-695f51
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat with his planning session (hc t4c), 2026-11-19 about 05:21Z, on the new acceptance criterion line for #1038's change in ss-lead/gates/1093-1038-criterion.md ('New'); relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-19T14:08:38Z · advance · plan → build · native:approved ^t-61e7a0
  - native_change: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
  - gate: native:approved
  - revision: b8c35abe01826be739ad0b6022ee3449263db889
  - by: claude-code (hc t59)
