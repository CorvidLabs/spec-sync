---
kind: session
id: 2026-11-06-s-late-close
aliases: [s-late-close]
objective: "specsync doctor: report every change problem with its fix; close merged-but-unarchived changes late into .specsync/archive/closed-late/, which 6.0-6.2 never read (#833, SHIP-12, RECOVER-3)"
intent_scope: SHIP
modules: [change, cmd_change, cli_args]
stage: test
stage_status: archived
iteration: 1
native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
criteria:
  - SHIP-12
  - SHIP-12.a
  - SHIP-12.b
  - SHIP-12.c
  - SHIP-12.d
  - SHIP-12.e
  - SHIP-12.f
  - SHIP-12.g
  - RECOVER-3
  - RECOVER-3.a
  - RECOVER-3.b
  - RECOVER-3.c
  - RECOVER-3.d
requirements: [REQ-change-143, REQ-cmd-change-035, REQ-cli-args-025]
observations:
  - OBS-product-test-8a9f5a47598253cb2208e85c
  - OBS-product-test-943d52f230af9865e7c1fe58
  - OBS-product-test-d753cf8983ee0af03f0458f9
  - OBS-product-test-a44413a32a903e5b908007e8
bugs: [BUG-late-close-follows-destination-symlinks-outside-the-project-69c842b5326c22b0c1dbafd3]
closed: 2026-11-13
outcome: done
---
# specsync doctor: report every change problem with its fix; close merged-but-unarchived changes late into .specsync/archive/closed-late/, which 6.0-6.2 never read (#833, SHIP-12, RECOVER-3)

## Goal

specsync doctor: report every change problem with its fix; close merged-but-unarchived changes late into .specsync/archive/closed-late/, which 6.0-6.2 never read (#833, SHIP-12, RECOVER-3)

## Where it stands

- Proof: real 6.0.0, 6.0.1, 6.1.0, 6.1.1, 6.1.2, 6.2.0 and main ignore .specsync/archive/closed-late/ (synthetic fixture and site clone); limits L1-L4 recorded in research.md
- Re-approved at 3a15acb2; main merged at 0ec91f3a; materialized at 7fc935c7; verify green at 64fe846e
- Shipped: archived at f56fb483 with no commit between review and ship
- Main merged at f3e0ca86 (#995 relay repair): change.3md #833 plane renumbered to z=28 and a stray separator line from merge 05171556 removed; build, change audit --strict and pre-push green
- Test-first #994 P1: red commit 9532a256b4b2e6c39fee88e39ebb212bb3125e18; all five new regressions failed, then focused fixed-code run passed 4 unit and 11 integration tests.
- Recovery refinement failed first at test commit 8ad024f86f52572c95c679f801a648f199b53a9b, then all five P1 regressions passed including real/dangling final-component retry. Final local and hosted gates still pending; missing Attest notes reported to orc t19f.

## Decisions and why

- Leif via orc t18 ('Capture all + approve design', 2026-11-06): capture SHIP-12 and .a-.f verbatim; D1 closed_late is a new terminal state (rejected: archived plus a marker, which could pass as verified); D2 drafts closable, recorded never approved; D3 several IDs all-or-nothing plus --dry-run (--all-merged deferred); D4 this PR first; it Fixes #833; the reopen overflow is #976 in its own PR (REQ-change-144).
- Leif via orc t18 (new direction, 2026-11-06): build it as specsync doctor; the late close is one doctor fix stored where 6.x never reads; ship in 6.3.0 with no new state (rejected: closed_late ChangeState, which 6.0-6.2 cannot parse; rejected: top-level .specsync/closed-late/, which 6.x treats as a project input).
- Leif via orc t18 ('Capture + recommended design'): capture SHIP-12.g and RECOVER-3 .a-.d verbatim; D2 stale verification is report-only; D3 --close-late <id>... or all with --actor/--reason; D4 the 6.0.0/6.2.0 compat test runs in CI; D5 a collision member drops and records its acknowledgement; IDs REQ-change-143 (reworded), REQ-cmd-change-035, REQ-cli-args-025. Remove the never-approved close-late draft.
- 0xLeif decided "Leif's scope approval of the doctor change, relayed by orc t18": "Leif approved specsync doctor; recorded as 0xLeif at 2e65c8a8" (orc t18 relay, 2026-11-06); recorded by claude (t-ss worker); a relayed claim, not authenticated
- Leif via orc t18 (option A, 2026-11-06): send-back recorded as 0xLeif after change ship refused with 'acceptance input src/agents.rs is production source without deterministic canonical ownership' (the approved affected_paths named all of src/); narrow affected_paths to the touched files and add cli (owner of src/main.rs) with a Purpose delta; the cancelled review stays in history. Rejected: 127 owner corrections for files the change never touched; --all-missing (one module for every file is a false ownership claim).
- 0xLeif decided "Leif's re-approval of the re-scoped doctor change, relayed by orc t18": "#994 re-approval recorded as 0xLeif at 3a15acb2, including the wording changes from the three P2 fixes" (orc t18 relay, 2026-11-06); recorded by claude (t-ss worker); a relayed claim, not authenticated
- Leif's scoped review of #994 after the re-scope: pass, recorded as 0xLeif by orc t18 at c0e905dd ('#994 fresh review recorded as 0xLeif, pass at c0e905dd. Ship, push the archive tip, watch the new compat CI job, then queue')
- Agent scope confirmed by orc t19f on 2026-11-06: repair only the destination-symlink P1 on existing archived-tip PR #994 under SHIP-12/REQ-change-143; no new native change or IDs, and archived packages remain immutable. Fresh Leif review is required at final full SHA.
- Agent decision: reuse lifecycle-lock component checks because create_dir_all follows links and exists misses dangling links; revalidate every plan before writes and again before each rename. No descriptor-relative concurrent-writer guarantee is claimed.

## What was verified

- Historical red proof for #994: test-only commit 9532a256b4b2e6c39fee88e39ebb212bb3125e18 on ba934e74ef8997fea35ca6e1f3987b10e7c68379 implementation; write-boundary unit failed and all four CLI regressions failed. Linked ancestor/leaf destinations move source packages outside; dangling links pass dry-run. — 543adef8c4df ^2026-11-07-historical-red-proof-for-994-test-only-commit
- Focused fixed-code run: fledge run test -- late_close_ -- --nocapture, pinned Rust 1.89.0, CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 nice -n 10; 4 unit and 11 integration tests passed, including all five new destination-symlink regressions, refusal recovery, unchanged outside/source bytes and existing rollback controls. — 342ca151390e ^2026-11-07-focused-fixed-code-run-fledge-run-test-late-close-nocapture
- Test-first recovery refinement on 082f4c7377565db465b6ffba64991182d6958c62: the write-boundary matrix fails because its final-component refusal says to create an in-project directory, which would block retry. The tightened test requires leaving the dated destination absent, then executes that retry for real and dangling final links. — bd90d9880244 ^2026-11-07-test-first-recovery-refinement-on
- All five P1 regressions pass after recovery refinement: 1 write-boundary unit matrix plus 4 CLI tests. The matrix now follows the final-component refusal by removing only the symlink, then closes both plans byte-for-byte with verified false and unchanged outside bytes. Linked/dangling parent dry-run/write refusal and parent recovery remain green. — f31f29b3ebff ^2026-11-07-all-five-p1-regressions-pass-after-recovery-refinement-1-write

## Bugs

- [Late close follows destination symlinks outside the project](../../bugs/2026-11-07-late-close-follows-destination-symlinks-outside-the-project.md): verified

## Open threads

## Timeline

- 2026-11-06T19:10:49Z · gate-opened · intent → intent · Leif's scope approval of the close-late change, relayed by orc t18 ^t-ff28af
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: f7180c3481814c7b05671fb413b1ffec5de829e2
- 2026-11-06T20:15:49Z · gate-closed · intent → intent · Leif's scope approval of the close-late change, relayed by orc t18 ^t-e17679
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 1276e314790521904552e375a659ae3214aa04dc
- 2026-11-06T20:15:49Z · gate-opened · intent → intent · Leif's scope approval of the doctor change, relayed by orc t18 ^t-dc9528
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 1276e314790521904552e375a659ae3214aa04dc
- 2026-11-06T23:41:26Z · gate-closed · intent → intent · Leif's scope approval of the doctor change, relayed by orc t18 ^t-de57d3
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 8b85e425dd7ea158e4584c3245e78df0d0788c25
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif approved specsync doctor; recorded as 0xLeif at 2e65c8a8
    - where: orc t18 relay, 2026-11-06
    - recorded_by: claude (t-ss worker)
- 2026-11-06T23:41:33Z · advance · intent → design ·  ^t-c07ff2
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 8b85e425dd7ea158e4584c3245e78df0d0788c25
- 2026-11-06T23:41:33Z · advance · design → plan ·  ^t-9549af
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 8b85e425dd7ea158e4584c3245e78df0d0788c25
- 2026-11-06T23:41:36Z · advance · plan → build · native:verifying ^t-2e2e2c
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - gate: native:verifying
  - revision: 8b85e425dd7ea158e4584c3245e78df0d0788c25
- 2026-11-06T23:41:39Z · advance · build → test · native:verifying ^t-8ce705
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - gate: native:verifying
  - revision: 8b85e425dd7ea158e4584c3245e78df0d0788c25
- 2026-11-07T00:15:08Z · gate-opened · test → test · Leif's re-approval of the re-scoped doctor change, relayed by orc t18 ^t-84b53c
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 457724521b380732b1ea8820bb0fdc81e59d7670
- 2026-11-07T00:45:29Z · gate-closed · test → test · Leif's re-approval of the re-scoped doctor change, relayed by orc t18 ^t-7057fc
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 64fe846e3b354ae9d34545883b46dc99b3bd9f7d
  - relayed_by:
    - decided_by: 0xLeif
    - words: #994 re-approval recorded as 0xLeif at 3a15acb2, including the wording changes from the three P2 fixes
    - where: orc t18 relay, 2026-11-06
    - recorded_by: claude (t-ss worker)
- 2026-11-07T06:51:41Z · gate-opened · test → test · Leif review of #994 at the final full SHA after local and hosted gates; old review does not cover this P1 implementation ^t-9e8d47
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 9532a256b4b2e6c39fee88e39ebb212bb3125e18
- 2026-11-13T04:45:37Z · gate-closed · test → test · Leif review of #994 at the final full SHA after local and hosted gates; old review does not cover this P1 implementation ^t-31e082
  - native_change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
  - revision: 131f31af6dd3c60b8b9f370ca7e67ac498f91158

Closed 2026-11-13: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-07-historical-red-proof-for-994-test-only-commit
    aliases: OBS-product-test-8a9f5a47598253cb2208e85c
    claim: "Historical red proof for #994: test-only commit 9532a256b4b2e6c39fee88e39ebb212bb3125e18 on ba934e74ef8997fea35ca6e1f3987b10e7c68379 implementation; write-boundary unit failed and all four CLI regressions failed. Linked ancestor/leaf destinations move source packages outside; dangling links pass dry-run."
    result: failed
    revision: 9532a256b4b2e6c39fee88e39ebb212bb3125e18
    environment: local-macos-arm64
    content: 0077e9b4ea007f052780fd94a72ffb328f842dac0bcaa7ea970301d71212dc9a
    fingerprint: sha256:543adef8c4df0f0c645d9a66e2d01d8cb1883f591e337ffe43ee54e6fe00da1d
    observed: 2026-11-07T06:51:18Z
  - id: 2026-11-07-focused-fixed-code-run-fledge-run-test-late-close-nocapture
    aliases: OBS-product-test-943d52f230af9865e7c1fe58
    claim: "Focused fixed-code run: fledge run test -- late_close_ -- --nocapture, pinned Rust 1.89.0, CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 nice -n 10; 4 unit and 11 integration tests passed, including all five new destination-symlink regressions, refusal recovery, unchanged outside/source bytes and existing rollback controls."
    result: passed
    revision: 9532a256b4b2e6c39fee88e39ebb212bb3125e18
    environment: local-macos-arm64
    content: 0077e9b4ea007f052780fd94a72ffb328f842dac0bcaa7ea970301d71212dc9a
    fingerprint: sha256:342ca151390ea4435a30150af0e373b2bb9e5d0a7aae229af6d7f4055e8bc4bc
    observed: 2026-11-07T06:51:19Z
  - id: 2026-11-07-test-first-recovery-refinement-on
    aliases: OBS-product-test-d753cf8983ee0af03f0458f9
    claim: "Test-first recovery refinement on 082f4c7377565db465b6ffba64991182d6958c62: the write-boundary matrix fails because its final-component refusal says to create an in-project directory, which would block retry. The tightened test requires leaving the dated destination absent, then executes that retry for real and dangling final links."
    result: failed
    revision: 082f4c7377565db465b6ffba64991182d6958c62
    environment: local-macos-arm64
    content: d7f5e8209916152a3fa751103616c7e518c555355bb66c03959c598babe43cd6
    fingerprint: sha256:bd90d9880244aadcd1cec3309f42624c4cc493abf16cb01f52992ddc96f25363
    observed: 2026-11-07T06:57:22Z
  - id: 2026-11-07-all-five-p1-regressions-pass-after-recovery-refinement-1-write
    aliases: OBS-product-test-a44413a32a903e5b908007e8
    claim: "All five P1 regressions pass after recovery refinement: 1 write-boundary unit matrix plus 4 CLI tests. The matrix now follows the final-component refusal by removing only the symlink, then closes both plans byte-for-byte with verified false and unchanged outside bytes. Linked/dangling parent dry-run/write refusal and parent recovery remain green."
    result: passed
    revision: 8ad024f86f52572c95c679f801a648f199b53a9b
    environment: local-macos-arm64
    content: f212182c825c7091fdf8d90b9733a880b3b8886def7c3a40bad380e674620c39
    fingerprint: sha256:f31f29b3ebff971ba1bc8975396490d80243dfcef9c2521d750559b5eb6a6723
    observed: 2026-11-07T06:58:49Z
```
