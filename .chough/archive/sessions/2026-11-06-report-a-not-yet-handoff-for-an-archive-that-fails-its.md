---
kind: session
id: 2026-11-06-report-a-not-yet-handoff-for-an-archive-that-fails-its
aliases:
  - report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
objective: Corrupt archive must not report a safe handoff (#880, audit D3)
intent_scope: CHANGE
modules: [change, cmd_change]
stage: test
stage_status: active
iteration: 1
callers: []
native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
pr: ""
criteria: [CHANGE-7.a, CHANGE-7.a.1, CHANGE-7.a.2]
requirements: [REQ-change-093, REQ-cmd-change-014]
cases: []
design: []
files: []
observations:
  - OBS-product-test-03accb387b007d6f0caf0ac6
  - OBS-product-test-167c1dc7ef56460da9f79442
  - OBS-product-test-779bc49d16e9a58a6685d19a
  - OBS-product-test-428ee81e19dd07f4e73779c9
  - OBS-product-test-fa3e114c08086d98d0991461
  - OBS-product-test-55d127983926e7a4dd727da6
  - OBS-product-test-8675bce722e7427d2e9712ad
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: check --commit, audit --strict, pre-push, push; orc records Leif's review; change ship with no commit between; push archive tip; CI; merge on my queue turn
closed: 2026-11-06
outcome: closed-without-done
---
# Corrupt archive must not report a safe handoff (#880, audit D3)

## Goal

Corrupt archive must not report a safe handoff (#880, audit D3)

## Where it stands

- Reproduced on main ad50bdef with the audit fixture: JSON handoff safe beside corrupt-history; text Next 'no further action' and Handoff safe; the named tail 'specsync change check' prints 'Nothing to check'
- Prototype (reverted, patches in scratchpad): git restore named command recovers tampered archive; committed damage recovers from the archiving commit and stays authenticated after commit
- Implemented (variant A): CorruptArchive signal, not-yet handoff, one restore step for handoff/JSON next/text Next/ship-status; red then green; auditor fixture recovers; perf follow-up #897 filed
- fledge lanes run verify green on merged tree (pinned 1.89): 2825 unit + 484 integration, strict specs 62/62
- Codex P1/P2a fixed test-first (2 new unit tests red on db2e3ab4, green now); P2b filed as #905
- P1 red first: warm_legacy_baseline_introductions_do_not_grow_with_legacy_archives failed at 1034b79b (19 vs 67 Git processes for 2 vs 8 legacy archives); green with the cache, plus a_cached_legacy_introduction_pass_never_hides_a_changed_archive_or_baseline; 79 related unit tests green
- Measured on spec-sync (44 legacy archives, release build): warm text status of CHG-0001 458 -> 271 Git processes, 0 introduction walks; remaining 144 are the workflow-v1 walks S9 (#918) removes
- Merged main through #944 (#914 fix) at 48a870a9: integration file rebuilt from main plus #880's test; lessons z=19 (change) and z=7 (cmd_change), invariant 49; both integration tests and 13 targeted unit tests green

## Decisions and why

- Agent finding, not a human decision: text status on an authenticated archive here costs 0.55s today, ~10.7s with the JSON-equal check (variant A), ~3.7s with an own-package-only check (variant C); the extra ~7s is the project roster walk (73 rev-list + 73 log over legacy CHG archives) the persistent read cache does not absorb
- Agent: per-change Chough session per orc t18 coordination; shared session.json adopted byte-for-byte under the #861 change key
- Leif (2026-11-05 20:42 MST, orc tab, relayed by orc t18): intent "Approve" for CHANGE-7.a.1 and CHANGE-7.a.2 as worded; scope "A + perf follow-up"; native definition approval recorded as 0xLeif at 7e6192a9
- rejected: variant C (own-package-only text check, ~3.7s) because text could disagree with JSON when the roster check fails, which is the D3 defect class
- Leif (2026-11-05 22:07 MST, orc tab, relayed by orc t18): "Record and merge" for #901 at db2e3ab4; superseded the same evening when orc t18 directed option 1 under the standing rule: fix the Codex P1 (unquoted shell words) and P2a (no-op package restore) in #901 test-first, file P2b
- decided: send-back as 'ss-s3 Claude agent' (orc t18 instruction) because answer is draft-only and the fix adds a third recovery case to the approved criteria and REQ-change-093
- rejected: exporting commands::merge::shell_quote because src/commands/merge.rs belongs to cmd_merge, outside the approved paths; a private one-line copy in the change domain instead
- Leif (2026-11-05, orc tab, relayed by orc t18): "Approve" re-approval of the sent-back scope (shell quoting + third recovery case); native definition approval recorded at 5abcc2c7
- Leif (2026-11-05 22:43 MST, standing rule relayed by orc t18): a PR he approved that only takes an origin/main merge (bookkeeping/changelog/version conflicts, reviewed content unchanged, checks green again) is merged at the new head without asking again, and the report says so; any change to reviewed content comes back for review
- decided: restore the shared .chough/session.json from origin/main and drop the byte-identical adopt copy because orc t18 asked each branch to commit only its own session file and history
- orc t18 relaying Leif (2026-11-06 morning): #901 first; merge origin/main, fix the Codex P1 test-first, defer every open P2 to its own issue (filed #928-#932, threads replied and resolved)
- decided: cache a passing legacy-baseline introductions proof persistently, keyed by baseline digest + full cutoff id + every legacy archive's current subtree digest, because the proof is a pure function of those and immutable commits, so no verdict changes
- rejected: a per-record targeted legacy check, because a legacy archive's verdict depends on the shared baseline context (one broken entry marks all legacy archives corrupt), so narrowing it would change verdicts and needs Leif
- Leif (via orc t18, 2026-11-06): #901 review OKed as pass at 5d169c9b, to be recorded once #914 merged; #914 merged as d0a98a0b (#944)
- Leif decided "Leif via orc t18: fresh scoped review of #901 after check --commit and verify; then ship with no commit between": "Leif OKs his #901 review (pass at 5d169c9b), to be recorded once #914 merges" (Herdr orc tab, 2026-11-06, relayed by orc t18; the review was recorded as 0xLeif at 30fb8ec0); recorded by ss-s3 Claude agent; a relayed claim, not authenticated

## What was verified

- Red first: integration test a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it with src at main ad50bdef (tree 7e6192a9 plus the test): JSON handoff.readiness 'safe' beside terminal_evidence 'corrupt-history' after tampering the archived workspace_digest ^2026-11-06-red-first-integration-test-a-corrupt-archive-is-not-safe-to
- Green: the same integration test passes with the fix (uncommitted tamper -> not-yet and the HEAD restore named in JSON, text Next/Handoff and ship-status; named command run verbatim -> authenticated-history and safe; committed tamper -> archiving-commit restore committed -> authenticated and safe); 24 focused unit tests pass (handoff*, corrupt_archive*, strict_check_reports_standalone_unprovable_archived_history, ship_status*, fast archived reads) ^2026-11-06-green-the-same-integration-test-passes-with-the-fix
- Auditor fixture fixtures/rust-main-v1 ('Archive greeting-version-1') with the fix's release binary: clean authenticated/safe; uncommitted tamper not-yet with HEAD restore; the named command restores to authenticated/safe; committed tamper not-yet with the archiving-commit form; restore from f49d652 committed -> authenticated/safe; clean tree ^2026-11-06-auditor-fixture-fixtures-rust-main-v1-archive-greeting-version
- Cost (release, macOS, this repo, 247 archives, authenticated #861 archive): text change status 0.55s -> ~10.7s on a quiet machine (prototype, same path); interleaved under load ~70: main 1.17/1.30/1.82s vs fix 18.2/19.8/21.7s; JSON unchanged (main 38.5/41.0/55.1s vs fix 38.7/40.7/67.4s); git processes for text status 24 -> 207 ^2026-11-06-cost-release-macos-this-repo-247-archives-authenticated-861
- fledge lanes run verify on the merged tree 025aead6 with the pinned 1.89 toolchain (PATH=<redacted:home-path> first): fmt, clippy -D warnings, check, cargo test 2825 unit + 484 integration passed (2 ignored), release build, strict specs 62/62, release-candidate-test 66 OK, ci-gate-test 35 OK; 23m54s ^2026-11-06-fledge-lanes-run-verify-on-the-merged-tree-025aead6-with-the
- Test-first for the Codex review of #901: corrupt_archive_restore_quotes_every_interpolated_word (sh could not parse the unquoted command for a hostile ID) and corrupt_archive_restore_names_the_package_only_when_the_package_changed (intact package got the no-op archiving-commit restore) failed on db2e3ab4 ^2026-11-06-test-first-for-the-codex-review-of-901-corrupt-archive-restore
- After the fix and merging origin/main (#889, #898) at d6a5c9d6: 26 focused unit tests (handoff*, corrupt_archive*, upstream_archive*, ship_status*, strict_check_reports_standalone_unprovable_archived_history) and 4 integration tests (a_corrupt_archive..., archived_upstream, status_prints_a_handoff_line) pass on pinned 1.89; the earlier full unit step on 5abcc2c7 passed 2827/2827 ^2026-11-06-after-the-fix-and-merging-origin-main-889-898-at-d6a5c9d6-26

## Open threads

## Timeline

- 2026-11-06T03:41:09Z · gate-opened · intent → intent · Leif via orc t18: confirm intent CHANGE-7.a.1/7.a.2 wording and approve scope of report-a-not-yet-handoff-... (variant A: text uses the JSON verdict, ~10.7s text status on archived here; or C: own-package only, ~3.7s) ^t-23bc48
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: ad50bdef3db58a715877877d2407485bcaed0f32
- 2026-11-06T03:43:46Z · gate-closed · intent → intent · Leif via orc t18: confirm intent CHANGE-7.a.1/7.a.2 wording and approve scope of report-a-not-yet-handoff-... (variant A: text uses the JSON verdict, ~10.7s text status on archived here; or C: own-package only, ~3.7s) ^t-7827f6
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
- 2026-11-06T03:43:47Z · advance · intent → design ·  ^t-2f2263
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
- 2026-11-06T03:43:47Z · skip · design → plan · CLI/JSON output change only; no visuals or interactions ^t-66227e
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
- 2026-11-06T03:43:52Z · advance · plan → build · native:approved ^t-548f71
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - gate: native:approved
  - revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
- 2026-11-06T04:59:28Z · advance · build → test · native:verifying ^t-2d8b9f
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - gate: native:verifying
  - revision: f244dee34bbd1516cab61f23bf4abd100e27942a
- 2026-11-06T04:59:28Z · gate-opened · test → test · Leif via orc t18: scoped implementation review of report-a-not-yet-handoff-... (PR for #880); then review + ship with no commit between ^t-70b596
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: f244dee34bbd1516cab61f23bf4abd100e27942a
- 2026-11-06T05:22:24Z · gate-closed · test → test · Leif via orc t18: scoped implementation review of report-a-not-yet-handoff-... (PR for #880); then review + ship with no commit between ^t-00f7de
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: db2e3ab4d52fdacfe97ca378c7f8b6dbdff64ef4
- 2026-11-06T05:22:24Z · gate-opened · test → test · Leif via orc t18: re-approve report-a-not-yet-handoff-... after the send-back (third recovery case + shell quoting), then a fresh scoped review ^t-4abcd4
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: db2e3ab4d52fdacfe97ca378c7f8b6dbdff64ef4
- 2026-11-06T05:25:14Z · gate-closed · test → test · Leif via orc t18: re-approve report-a-not-yet-handoff-... after the send-back (third recovery case + shell quoting), then a fresh scoped review ^t-7109e4
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: 5abcc2c78a5afaec76504f9c53e2a9482a07f57c
- 2026-11-06T05:25:15Z · gate-opened · test → test · Leif via orc t18: fresh scoped review of #901 after check --commit and verify; then ship with no commit between ^t-84db61
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: 5abcc2c78a5afaec76504f9c53e2a9482a07f57c
- 2026-11-06T23:34:39Z · gate-closed · test → test · Leif via orc t18: fresh scoped review of #901 after check --commit and verify; then ship with no commit between ^t-7d8165
  - native_change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
  - revision: 4d1febe1cd34a4365829026657b691a2a0704fa6
  - relayed_by:
    - decided_by: Leif
    - words: Leif OKs his #901 review (pass at 5d169c9b), to be recorded once #914 merges
    - where: Herdr orc tab, 2026-11-06, relayed by orc t18; the review was recorded as 0xLeif at 30fb8ec0
    - recorded_by: ss-s3 Claude agent

Closed 2026-11-06: closed-without-done.

## Receipts

```yaml
receipts:
  - id: 2026-11-06-red-first-integration-test-a-corrupt-archive-is-not-safe-to
    aliases: OBS-product-test-03accb387b007d6f0caf0ac6
    claim: "Red first: integration test a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it with src at main ad50bdef (tree 7e6192a9 plus the test): JSON handoff.readiness 'safe' beside terminal_evidence 'corrupt-history' after tampering the archived workspace_digest"
    result: failed
    revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
    environment: local
    content: 1b64933cffc44ec6785b3f9fe81797c2ff3edd62409bf045e706d7894f05ec51
    observed: 2026-11-06T04:28:01Z
  - id: 2026-11-06-green-the-same-integration-test-passes-with-the-fix
    aliases: OBS-product-test-167c1dc7ef56460da9f79442
    claim: "Green: the same integration test passes with the fix (uncommitted tamper -> not-yet and the HEAD restore named in JSON, text Next/Handoff and ship-status; named command run verbatim -> authenticated-history and safe; committed tamper -> archiving-commit restore committed -> authenticated and safe); 24 focused unit tests pass (handoff*, corrupt_archive*, strict_check_reports_standalone_unprovable_archived_history, ship_status*, fast archived reads)"
    result: passed
    revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
    environment: local
    content: 1b64933cffc44ec6785b3f9fe81797c2ff3edd62409bf045e706d7894f05ec51
    observed: 2026-11-06T04:28:02Z
  - id: 2026-11-06-auditor-fixture-fixtures-rust-main-v1-archive-greeting-version
    aliases: OBS-product-test-779bc49d16e9a58a6685d19a
    claim: "Auditor fixture fixtures/rust-main-v1 ('Archive greeting-version-1') with the fix's release binary: clean authenticated/safe; uncommitted tamper not-yet with HEAD restore; the named command restores to authenticated/safe; committed tamper not-yet with the archiving-commit form; restore from f49d652 committed -> authenticated/safe; clean tree"
    result: passed
    revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
    environment: local
    content: 1b64933cffc44ec6785b3f9fe81797c2ff3edd62409bf045e706d7894f05ec51
    observed: 2026-11-06T04:28:05Z
  - id: 2026-11-06-cost-release-macos-this-repo-247-archives-authenticated-861
    aliases: OBS-product-test-428ee81e19dd07f4e73779c9
    claim: "Cost (release, macOS, this repo, 247 archives, authenticated #861 archive): text change status 0.55s -> ~10.7s on a quiet machine (prototype, same path); interleaved under load ~70: main 1.17/1.30/1.82s vs fix 18.2/19.8/21.7s; JSON unchanged (main 38.5/41.0/55.1s vs fix 38.7/40.7/67.4s); git processes for text status 24 -> 207"
    result: passed
    revision: 7e6192a9ec4d162911f415316a641f9eafad0c20
    environment: local
    content: 1b64933cffc44ec6785b3f9fe81797c2ff3edd62409bf045e706d7894f05ec51
    observed: 2026-11-06T04:28:07Z
  - id: 2026-11-06-fledge-lanes-run-verify-on-the-merged-tree-025aead6-with-the
    aliases: OBS-product-test-fa3e114c08086d98d0991461
    claim: "fledge lanes run verify on the merged tree 025aead6 with the pinned 1.89 toolchain (PATH=<redacted:home-path> first): fmt, clippy -D warnings, check, cargo test 2825 unit + 484 integration passed (2 ignored), release build, strict specs 62/62, release-candidate-test 66 OK, ci-gate-test 35 OK; 23m54s"
    result: passed
    revision: 025aead6ec63cd1ef7492d0325957f6c27504475
    environment: local
    content: 2bbbd5f8519d5d1973e76ff10365285b9b5aeb68519024ac341f6138da8bf577
    observed: 2026-11-06T04:58:52Z
  - id: 2026-11-06-test-first-for-the-codex-review-of-901-corrupt-archive-restore
    aliases: OBS-product-test-55d127983926e7a4dd727da6
    claim: "Test-first for the Codex review of #901: corrupt_archive_restore_quotes_every_interpolated_word (sh could not parse the unquoted command for a hostile ID) and corrupt_archive_restore_names_the_package_only_when_the_package_changed (intact package got the no-op archiving-commit restore) failed on db2e3ab4"
    result: failed
    revision: d6a5c9d6c52f96353eacdc27b5e7b4f30fdb84a3
    environment: local
    content: 5006a763150def07080baa83455495bfb13e70cce39e0cf833241f7b7f36a29b
    observed: 2026-11-06T05:51:34Z
  - id: 2026-11-06-after-the-fix-and-merging-origin-main-889-898-at-d6a5c9d6-26
    aliases: OBS-product-test-8675bce722e7427d2e9712ad
    claim: "After the fix and merging origin/main (#889, #898) at d6a5c9d6: 26 focused unit tests (handoff*, corrupt_archive*, upstream_archive*, ship_status*, strict_check_reports_standalone_unprovable_archived_history) and 4 integration tests (a_corrupt_archive..., archived_upstream, status_prints_a_handoff_line) pass on pinned 1.89; the earlier full unit step on 5abcc2c7 passed 2827/2827"
    result: passed
    revision: d6a5c9d6c52f96353eacdc27b5e7b4f30fdb84a3
    environment: local
    content: 5006a763150def07080baa83455495bfb13e70cce39e0cf833241f7b7f36a29b
    observed: 2026-11-06T05:51:38Z
```
