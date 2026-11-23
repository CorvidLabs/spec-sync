---
kind: session
id: 2026-11-20-state-outside-the-work-tree
aliases: [state-outside-the-work-tree]
objective: SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules: [util, hash_cache, change, cmd_check, rehash, cmd_resolve, hooks, validator]
stage: done
stage_status: archived
iteration: 1
native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
observations:
  - 2026-11-20-on-a2a50824-with-only-change-6-s-tests-added-b2c70e1f-all-5
  - 2026-11-20-with-the-fix-at-0187d498-fledge-lanes-run-verify-passes-all-10
  - 2026-11-20-on-linux-at-0187d498-cargo-clippy-all-targets-d-warnings
bugs: [2026-11-20-state-a-repository-ships-is-trusted]
closed: 2026-11-20
outcome: done
---
# SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)

## Goal

SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree": "a, all yes.. im back" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 00:38Z, answering item 1 as t4c put it at about 21:09Z on 2026-11-19 with option A: 'one more change (change 6): move the hash cache, journal and remote cache out of the work tree; close the scan blind spots, the hooks-folder case and the enclosing-repo case; leave the paths you type yourself (--template) as they are, and document the low-rated items. Then this same reviewer re-checks its own findings, rather than a third full review. [...] (fold [change 5's review] into change 6 if you pick A)', and that the class fix moves them 'into the Git directory, where a repo can't commit files, or into your user cache folder when there's no Git'; t4c mapped it to review2's F1, F2, F3, F4, F6 and F7 fixed, F5 kept and documented, F8, F9 and F10 documented; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve keep-specsync-s-own-caches-and-journal-outside-the-work-tree as 0xLeif
- 0xLeif decided "Implementation review of keep-specsync-s-own-caches-and-journal-outside-the-work-tree": "Pass (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 06:04Z, answering 'Security fix, your one review of changes 5-8. ... The reviewer says your bar is MET ... Confidence 75% that the fix is complete, 90% that the bar holds. What's left is 13 low items for 6.4.1. ... Does your review pass?' with 'Pass (Recommended)'; under his earlier answer (about 04:16Z) 'Yes, one review (Recommended)' to 'Does your one review cover all four [changes 5-8], each shipped in order as long as its own lines haven't changed since you reviewed?'. Reviewed tip e843776a; this change's own lines are unchanged since (only its native check, which materializes its deltas, ran after the earlier change's archive). Relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change review keep-specsync-s-own-caches-and-journal-outside-the-work-tree as 0xLeif with verdict pass

## What was verified

- On a2a50824 with only change 6's tests added (b2c70e1f): all 5 new unit tests and 8 of 13 new integration tests fail (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-cmd-resolve-002). A forged pass in .specsync/hashes.json replays in an unpacked tree and, on this case-insensitive disk, when committed as .specsync/Hashes.json; check in an unpacked copy skips the spec on a shipped cache; a valid journal planted in the work tree is replayed by recover and by change new, with or without Git; a running transaction writes its journal into the work tree; a committed .specsync-cache/remote-specs entry is returned by the remote cache; hooks install --precommit with core.hooksPath in the work tree created outside/hooks through a linked .config and outside/created through a dangling pre-commit link; an untracked project in a home folder kept under Git appended to the outer notes.md. The read scan finds 12 unmarked reads (src/validator/tests.rs:3452 among them), the write scan 7, the Git scan 5 (git worktree add/remove/prune). The 5 that pass are controls: three monorepo subproject tests, a real in-tree hooks folder, and the lexer test. - 85ef1dd11116 ^2026-11-20-on-a2a50824-with-only-change-6-s-tests-added-b2c70e1f-all-5
- With the fix at 0187d498, fledge lanes run verify passes all 10 steps: unit 3122 passed, 0 failed, 2 ignored; integration 697 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 5 new unit tests and 13 new integration tests pass (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-rehash-003, REQ-cmd-resolve-002, REQ-cmd-resolve-003), the scans find nothing unmarked, and the older tests that assumed .specsync/hashes.json and the in-tree journal now point at the local state folder. - 9773294cafec ^2026-11-20-with-the-fix-at-0187d498-fledge-lanes-run-verify-passes-all-10
- On Linux at 0187d498: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3126 passed, 0 failed, 2 ignored and integration 700 passed, 0 failed. The first Linux run at 5a449fb0 found two defects macOS could not show, both fixed before this run: fsync on the O_PATH folder handle failed every durable journal write (fixed in 1a5adcdd), and a clean cargo test writes no dep-info beside the binary, so the scans could not find their file list (fixed in 93e06a8e by reading the deps/ dep-info of the exact binary). Not run: x86_64, the Ubuntu image CI uses, hosted CI. - f9606bbc736b ^2026-11-20-on-linux-at-0187d498-cargo-clippy-all-targets-d-warnings

## Bugs

- [State a repository ships is trusted](../../bugs/2026-11-20-state-a-repository-ships-is-trusted.md): verified

## Open threads

## Timeline

- 2026-11-20T00:50:57Z · start · intent → intent · SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54) ^t-afb3b0
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T00:55:40Z · advance · intent → design ·  ^t-d4f09c
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T00:55:50Z · advance · design → plan ·  ^t-109984
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T00:55:50Z · gate-opened · plan → plan · Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree ^t-3dc27e
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T00:56:19Z · gate-closed · plan → plan · Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree ^t-89692a
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - relayed_by:
    - decided_by: 0xLeif
    - words: a, all yes.. im back
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 00:38Z, answering item 1 as t4c put it at about 21:09Z on 2026-11-19 with option A: 'one more change (change 6): move the hash cache, journal and remote cache out of the work tree; close the scan blind spots, the hooks-folder case and the enclosing-repo case; leave the paths you type yourself (--template) as they are, and document the low-rated items. Then this same reviewer re-checks its own findings, rather than a third full review. [...] (fold [change 5's review] into change 6 if you pick A)', and that the class fix moves them 'into the Git directory, where a repo can't commit files, or into your user cache folder when there's no Git'; t4c mapped it to review2's F1, F2, F3, F4, F6 and F7 fixed, F5 kept and documented, F8, F9 and F10 documented; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve keep-specsync-s-own-caches-and-journal-outside-the-work-tree
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T00:56:57Z · advance · plan → build · native:approved ^t-ef6ef1
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - gate: native:approved
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T06:15:56Z · direct-native-write · build → build · keep-specsync-s-own-caches-and-journal-outside-the-work-tree's lifecycle records changed outside Chough: state.json changed, verification.json removed, verification-attempts.json removed; state verifying → approved ^t-dd0c7e
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: 33f018109b44f5a8e8d37c46d8aebe9ad1fb76de
  - direct_write:
    - change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
    - files:
      - state.json: changed
      - verification.json: removed
      - verification-attempts.json: removed
    - before_state: verifying
    - after_state: approved
  - by: claude-code (hc t5r)
- 2026-11-20T06:15:56Z · advance · build → test · native:verifying ^t-681a03
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - gate: native:verifying
  - revision: 33f018109b44f5a8e8d37c46d8aebe9ad1fb76de
  - by: claude-code (hc t5r)
- 2026-11-20T06:15:59Z · advance · test → accept · native:verifying ^t-b1f6db
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - gate: native:verifying
  - revision: 33f018109b44f5a8e8d37c46d8aebe9ad1fb76de
  - by: claude-code (hc t5r)
- 2026-11-20T06:16:27Z · gate-opened · accept → accept · Implementation review of keep-specsync-s-own-caches-and-journal-outside-the-work-tree ^t-3fa0f2
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: 33f018109b44f5a8e8d37c46d8aebe9ad1fb76de
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
- 2026-11-20T06:21:10Z · gate-closed · accept → accept · Implementation review of keep-specsync-s-own-caches-and-journal-outside-the-work-tree ^t-dbce1f
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: 146be4432c1e2ff2bc6ab4ba17a3ffb489a4b886
  - relayed_by:
    - decided_by: 0xLeif
    - words: Pass (Recommended)
    - where: Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 06:04Z, answering 'Security fix, your one review of changes 5-8. ... The reviewer says your bar is MET ... Confidence 75% that the fix is complete, 90% that the bar holds. What's left is 13 low items for 6.4.1. ... Does your review pass?' with 'Pass (Recommended)'; under his earlier answer (about 04:16Z) 'Yes, one review (Recommended)' to 'Does your one review cover all four [changes 5-8], each shipped in order as long as its own lines haven't changed since you reviewed?'. Reviewed tip e843776a; this change's own lines are unchanged since (only its native check, which materializes its deltas, ran after the earlier change's archive). Relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change review keep-specsync-s-own-caches-and-journal-outside-the-work-tree
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T06:21:14Z · advance · accept → done · native:archived ^t-ee340c
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - gate: native:archived
  - revision: 146be4432c1e2ff2bc6ab4ba17a3ffb489a4b886
  - by: claude-code (hc t5r)

Closed 2026-11-20: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-20-on-a2a50824-with-only-change-6-s-tests-added-b2c70e1f-all-5
    claim: "On a2a50824 with only change 6's tests added (b2c70e1f): all 5 new unit tests and 8 of 13 new integration tests fail (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-cmd-resolve-002). A forged pass in .specsync/hashes.json replays in an unpacked tree and, on this case-insensitive disk, when committed as .specsync/Hashes.json; check in an unpacked copy skips the spec on a shipped cache; a valid journal planted in the work tree is replayed by recover and by change new, with or without Git; a running transaction writes its journal into the work tree; a committed .specsync-cache/remote-specs entry is returned by the remote cache; hooks install --precommit with core.hooksPath in the work tree created outside/hooks through a linked .config and outside/created through a dangling pre-commit link; an untracked project in a home folder kept under Git appended to the outer notes.md. The read scan finds 12 unmarked reads (src/validator/tests.rs:3452 among them), the write scan 7, the Git scan 5 (git worktree add/remove/prune). The 5 that pass are controls: three monorepo subproject tests, a real in-tree hooks folder, and the lexer test."
    result: failed
    revision: b2c70e1fe315e15d2305563920bb6cba9ecb4550
    environment: macOS arm64 (APFS, case-insensitive), rustc 1.89.0, isolated git config, through one-lane; tree of b2c70e1f (a2a50824 + the tests only)
    content: 3633f56f0f0e3efe9e65bd00972f0125d37a593ef594dc3205214c762b8c8013
    fingerprint: sha256:85ef1dd11116389daeef88b5b24b2905406ba8a32bebabf4afa88ebc55d67c2a
    observed: 2026-11-20T01:12:01Z
    actor: claude-code (hc t5r)
  - id: 2026-11-20-on-linux-at-0187d498-cargo-clippy-all-targets-d-warnings
    claim: "On Linux at 0187d498: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3126 passed, 0 failed, 2 ignored and integration 700 passed, 0 failed. The first Linux run at 5a449fb0 found two defects macOS could not show, both fixed before this run: fsync on the O_PATH folder handle failed every durable journal write (fixed in 1a5adcdd), and a clean cargo test writes no dep-info beside the binary, so the scans could not find their file list (fixed in 93e06a8e by reading the deps/ dep-info of the exact binary). Not run: x86_64, the Ubuntu image CI uses, hosted CI."
    result: passed
    revision: 0187d49824c69d81cffd51d68b24bfd28afc6765
    environment: Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 0187d498, as a normal user with jq, through one-lane
    content: 2345cacabed0e5d381e05f4a2424e20aee042726356f24191ed679d284189a66
    fingerprint: sha256:f9606bbc736b09e76201a6eb3fb4339416bf2edc1e7443b3642803e410cc291c
    observed: 2026-11-20T02:51:25Z
    actor: claude-code (hc t5r)
  - id: 2026-11-20-with-the-fix-at-0187d498-fledge-lanes-run-verify-passes-all-10
    claim: "With the fix at 0187d498, fledge lanes run verify passes all 10 steps: unit 3122 passed, 0 failed, 2 ignored; integration 697 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 5 new unit tests and 13 new integration tests pass (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-rehash-003, REQ-cmd-resolve-002, REQ-cmd-resolve-003), the scans find nothing unmarked, and the older tests that assumed .specsync/hashes.json and the in-tree journal now point at the local state folder."
    result: passed
    revision: 0187d49824c69d81cffd51d68b24bfd28afc6765
    environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 0187d498 (change 6's code f6eac853, 1a5adcdd, 93e06a8e, with change 5 re-verified on it)
    content: 2345cacabed0e5d381e05f4a2424e20aee042726356f24191ed679d284189a66
    fingerprint: sha256:9773294cafecdc81155c0bd80b7e3b8640bf9ae5854af3b566af31254ec470e5
    observed: 2026-11-20T02:51:38Z
    actor: claude-code (hc t5r)
```
