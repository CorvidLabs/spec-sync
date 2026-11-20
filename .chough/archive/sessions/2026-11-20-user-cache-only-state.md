---
kind: session
id: 2026-11-20-user-cache-only-state
aliases: [user-cache-only-state]
objective: SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules: [util, change, hooks, git_utils, cmd_check]
stage: done
stage_status: archived
iteration: 1
native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
observations:
  - 2026-11-20-on-e3d508ac-with-only-change-7-s-tests-added-3fcb6de7-all-3
  - 2026-11-20-with-change-7-s-fix-at-97828091-fledge-lanes-run-verify-passes
  - 2026-11-20-on-linux-at-97828091-cargo-clippy-all-targets-d-warnings
bugs: [2026-11-20-state-an-archive-ships-in-its-git-directory-is-trusted]
closed: 2026-11-20
outcome: done
---
# SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)

## Goal

SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of keep-specsync-s-own-state-only-in-the-user-cache-folder": "A" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 03:27Z, answering t4c's question after sec-review2's re-check of change 6 (security/review2/recheck-change6.md): 'A (recommended): one more small change, about half a day. SpecSync keeps its cache only in your user cache folder, never in .git. That's simpler, one rule fewer, and closes the archive case and two related ones. It also fixes the one-line bug and the edge cases, then the reviewer does a quick re-check of just those. 6.4.0 moves to about Nov 25-26, and the security notice's claims stay true as written.' The one-line bug was R4 (a folder named * as a wildcard) and the edge cases R2, R5a and N2; t4c's brief lists items 1-5 (user cache folder only keyed by the canonical root; refuse a cache base inside the root or the confinement top; --literal-pathspecs and a clean environment in tracked_by_git; spec-shaped registry replay; a hooks path outside both trees only from global or system scope) and, as text and tests only, R6's small scan patterns and the documentation fixes, R5b documented; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve keep-specsync-s-own-state-only-in-the-user-cache-folder as 0xLeif
- 0xLeif decided "Scope re-approval of keep-specsync-s-own-state-only-in-the-user-cache-folder after the delta rewrite": "Yes, add them (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 14:07Z, answering 'Security changes 7 and 8: a SpecSync tooling quirk. After change 6 was archived, SpecSync won't re-approve change 7 unless its five requirement texts are "rewritten", even though they're already the final text you reviewed. The smallest rewrite it accepts adds a short clause to each, for example "This replaces the Git-directory location the previous text named." and ", now also,". No meaning or code changes, and the builder tested that approve and check pass with them. OK to add the five clauses so 7 and 8 can ship?'. The rewrite applied is exactly security/change7-delta-rebase.diff (five clauses in the util, change, hooks and cmd_check deltas; the git_utils delta unchanged), checked line for line. It rests on his scope answer 'A' (about 03:27Z) and his review 'Pass (Recommended)' (about 06:04Z) under 'Yes, one review (Recommended)' (about 04:16Z). Relayed by t4c over hc to t5r; a relayed claim, not authenticated); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve keep-specsync-s-own-state-only-in-the-user-cache-folder as 0xLeif
- 0xLeif decided "Implementation review of keep-specsync-s-own-state-only-in-the-user-cache-folder": "Pass (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 06:04Z, answering 'Security fix, your one review of changes 5-8. ... The reviewer says your bar is MET ... Confidence 75% that the fix is complete, 90% that the bar holds. What's left is 13 low items for 6.4.1. ... Does your review pass?' with 'Pass (Recommended)'; under his earlier answer (about 04:16Z) 'Yes, one review (Recommended)' to 'Does your one review cover all four [changes 5-8], each shipped in order as long as its own lines haven't changed since you reviewed?'. Reviewed tip e843776a. Since that review, this change's code and tests are unchanged; its deltas carry only the five clauses he approved at about 14:07Z ('Yes, add them (Recommended)', exactly security/change7-delta-rebase.diff). Relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change review keep-specsync-s-own-state-only-in-the-user-cache-folder as 0xLeif with verdict pass

## What was verified

- On e3d508ac with only change 7's tests added (3fcb6de7): all 3 new unit tests and 5 of 7 new integration tests fail (also REQ-git-utils-012, REQ-cmd-check-021). The state folder of a Git work tree is .git/specsync/<constant key>; a copy of a project with its .git replays the cache it ships (Skipped 1 unchanged spec, Cache: unpacked/.git/specsync/...); a valid journal at .git/specsync/<key>/change-transaction.json is replayed; a registry entry env = .env lets a journal delete .env; with XDG_CACHE_HOME inside the project the cache there is replayed; a project folder named * inside a home work tree that tracks Downloads/readme.txt is widened and the outer notes.md gets the CLAUDE.md block; a local core.hooksPath ../outside/made/hooks makes outside/made. The read scan finds 10 new unmarked opens (open_ambient_dir and .open(). The 2 that pass are the global-config hooks control and the write scan. - aa031ad59093 ^2026-11-20-on-e3d508ac-with-only-change-7-s-tests-added-3fcb6de7-all-3
- With change 7's fix at 97828091, fledge lanes run verify passes all 10 steps: unit 3125 passed, 0 failed, 2 ignored; integration 702 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 3 new unit tests and 7 new integration tests pass (also REQ-git-utils-012 and REQ-cmd-check-021); the state folder is in the user cache folder, a copy of a project with its .git validates every spec, a cache folder inside the project is refused, a folder named * stays confined, a registry entry naming .env opens no replay, a local core.hooksPath outside makes no folder, and the scans find nothing unmarked. - 5ae2654c32d3 ^2026-11-20-with-change-7-s-fix-at-97828091-fledge-lanes-run-verify-passes
- On Linux at 97828091: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3129 passed, 0 failed, 2 ignored and integration 705 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI. - 4c9675140e3d ^2026-11-20-on-linux-at-97828091-cargo-clippy-all-targets-d-warnings

## Bugs

- [State an archive ships in its git directory is trusted](../../bugs/2026-11-20-state-an-archive-ships-in-its-git-directory-is-trusted.md): verified

## Open threads

## Timeline

- 2026-11-20T03:30:14Z · start · intent → intent · SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54) ^t-57a76a
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - by: claude-code (hc t5r)
- 2026-11-20T03:30:14Z · advance · intent → design ·  ^t-81884a
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - by: claude-code (hc t5r)
- 2026-11-20T03:30:14Z · advance · design → plan ·  ^t-fc03e9
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - by: claude-code (hc t5r)
- 2026-11-20T03:32:35Z · gate-opened · plan → plan · Scope approval of keep-specsync-s-own-state-only-in-the-user-cache-folder ^t-56adfd
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T03:32:35Z · direct-native-write · plan → plan · keep-specsync-s-own-state-only-in-the-user-cache-folder's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-e1b08e
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - direct_write:
    - change: keep-specsync-s-own-state-only-in-the-user-cache-folder
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t5r)
- 2026-11-20T03:32:35Z · gate-closed · plan → plan · Scope approval of keep-specsync-s-own-state-only-in-the-user-cache-folder ^t-dfcd78
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - relayed_by:
    - decided_by: 0xLeif
    - words: A
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 03:27Z, answering t4c's question after sec-review2's re-check of change 6 (security/review2/recheck-change6.md): 'A (recommended): one more small change, about half a day. SpecSync keeps its cache only in your user cache folder, never in .git. That's simpler, one rule fewer, and closes the archive case and two related ones. It also fixes the one-line bug and the edge cases, then the reviewer does a quick re-check of just those. 6.4.0 moves to about Nov 25-26, and the security notice's claims stay true as written.' The one-line bug was R4 (a folder named * as a wildcard) and the edge cases R2, R5a and N2; t4c's brief lists items 1-5 (user cache folder only keyed by the canonical root; refuse a cache base inside the root or the confinement top; --literal-pathspecs and a clean environment in tracked_by_git; spec-shaped registry replay; a hooks path outside both trees only from global or system scope) and, as text and tests only, R6's small scan patterns and the documentation fixes, R5b documented; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve keep-specsync-s-own-state-only-in-the-user-cache-folder
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T03:35:05Z · advance · plan → build · native:approved ^t-67f9e8
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - gate: native:approved
  - revision: e3d508ac254dd94817409b30fe180423de027a87
  - by: claude-code (hc t5r)
- 2026-11-20T14:08:53Z · gate-opened · build → build · Scope re-approval of keep-specsync-s-own-state-only-in-the-user-cache-folder after the delta rewrite ^t-f2edd5
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: 54d80d0ac37e28fd8e294ce0d6ba1b56cef476e0
  - holds: build
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T14:08:53Z · gate-closed · build → build · Scope re-approval of keep-specsync-s-own-state-only-in-the-user-cache-folder after the delta rewrite ^t-91ced9
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: 54d80d0ac37e28fd8e294ce0d6ba1b56cef476e0
  - relayed_by:
    - decided_by: 0xLeif
    - words: Yes, add them (Recommended)
    - where: Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 14:07Z, answering 'Security changes 7 and 8: a SpecSync tooling quirk. After change 6 was archived, SpecSync won't re-approve change 7 unless its five requirement texts are "rewritten", even though they're already the final text you reviewed. The smallest rewrite it accepts adds a short clause to each, for example "This replaces the Git-directory location the previous text named." and ", now also,". No meaning or code changes, and the builder tested that approve and check pass with them. OK to add the five clauses so 7 and 8 can ship?'. The rewrite applied is exactly security/change7-delta-rebase.diff (five clauses in the util, change, hooks and cmd_check deltas; the git_utils delta unchanged), checked line for line. It rests on his scope answer 'A' (about 03:27Z) and his review 'Pass (Recommended)' (about 06:04Z) under 'Yes, one review (Recommended)' (about 04:16Z). Relayed by t4c over hc to t5r; a relayed claim, not authenticated
    - recorded_by: claude-code-t5r
    - native: specsync change approve keep-specsync-s-own-state-only-in-the-user-cache-folder
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T14:10:05Z · advance · build → test · native:verifying ^t-e58df3
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - gate: native:verifying
  - revision: 7943890ab12ee5c8dcbf394a4d3683ac2d1f3872
  - by: claude-code (hc t5r)
- 2026-11-20T14:10:11Z · advance · test → accept · native:verifying ^t-779dda
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - gate: native:verifying
  - revision: 7943890ab12ee5c8dcbf394a4d3683ac2d1f3872
  - by: claude-code (hc t5r)
- 2026-11-20T14:10:18Z · gate-opened · accept → accept · Implementation review of keep-specsync-s-own-state-only-in-the-user-cache-folder ^t-f6caf3
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: 7943890ab12ee5c8dcbf394a4d3683ac2d1f3872
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
- 2026-11-20T14:16:08Z · gate-closed · accept → accept · Implementation review of keep-specsync-s-own-state-only-in-the-user-cache-folder ^t-d5f2dc
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - revision: 5ffd449c3313dc2ee97d1062374209b610f3b9c0
  - relayed_by:
    - decided_by: 0xLeif
    - words: Pass (Recommended)
    - where: Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 06:04Z, answering 'Security fix, your one review of changes 5-8. ... The reviewer says your bar is MET ... Confidence 75% that the fix is complete, 90% that the bar holds. What's left is 13 low items for 6.4.1. ... Does your review pass?' with 'Pass (Recommended)'; under his earlier answer (about 04:16Z) 'Yes, one review (Recommended)' to 'Does your one review cover all four [changes 5-8], each shipped in order as long as its own lines haven't changed since you reviewed?'. Reviewed tip e843776a. Since that review, this change's code and tests are unchanged; its deltas carry only the five clauses he approved at about 14:07Z ('Yes, add them (Recommended)', exactly security/change7-delta-rebase.diff). Relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change review keep-specsync-s-own-state-only-in-the-user-cache-folder
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T14:16:12Z · advance · accept → done · native:archived ^t-bc94cf
  - native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
  - gate: native:archived
  - revision: 5ffd449c3313dc2ee97d1062374209b610f3b9c0
  - by: claude-code (hc t5r)

Closed 2026-11-20: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-20-on-e3d508ac-with-only-change-7-s-tests-added-3fcb6de7-all-3
    claim: "On e3d508ac with only change 7's tests added (3fcb6de7): all 3 new unit tests and 5 of 7 new integration tests fail (also REQ-git-utils-012, REQ-cmd-check-021). The state folder of a Git work tree is .git/specsync/<constant key>; a copy of a project with its .git replays the cache it ships (Skipped 1 unchanged spec, Cache: unpacked/.git/specsync/...); a valid journal at .git/specsync/<key>/change-transaction.json is replayed; a registry entry env = .env lets a journal delete .env; with XDG_CACHE_HOME inside the project the cache there is replayed; a project folder named * inside a home work tree that tracks Downloads/readme.txt is widened and the outer notes.md gets the CLAUDE.md block; a local core.hooksPath ../outside/made/hooks makes outside/made. The read scan finds 10 new unmarked opens (open_ambient_dir and .open(). The 2 that pass are the global-config hooks control and the write scan."
    result: failed
    revision: 3fcb6de7053366ab2b8d4a5246eaa9b247939e89
    environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tree of 3fcb6de7 (e3d508ac + change 7's records and tests only)
    content: f20ad66eda1be07d9d2700ab0f9fca4ae14fb03c7158babb6fd27547e7a325c2
    fingerprint: sha256:aa031ad590932c5c4641b641b0756a1c7a88afd0956c497984ea30a930f5281c
    observed: 2026-11-20T03:37:11Z
    actor: claude-code (hc t5r)
  - id: 2026-11-20-with-change-7-s-fix-at-97828091-fledge-lanes-run-verify-passes
    claim: "With change 7's fix at 97828091, fledge lanes run verify passes all 10 steps: unit 3125 passed, 0 failed, 2 ignored; integration 702 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 3 new unit tests and 7 new integration tests pass (also REQ-git-utils-012 and REQ-cmd-check-021); the state folder is in the user cache folder, a copy of a project with its .git validates every spec, a cache folder inside the project is refused, a folder named * stays confined, a registry entry naming .env opens no replay, a local core.hooksPath outside makes no folder, and the scans find nothing unmarked."
    result: passed
    revision: 97828091ad38ab5bdc7c4ab49f1d9554cfae8be3
    environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 97828091 (change 7's code 18d07160)
    content: 2c8e78ca4b990e9ef05fe352e68d6be0504a860db9de22e3267d1f77e5762688
    fingerprint: sha256:5ae2654c32d3deef0339bc2d1923acb1ee4682d911c105472e915f4dec0cc9a1
    observed: 2026-11-20T04:12:03Z
    actor: claude-code (hc t5r)
  - id: 2026-11-20-on-linux-at-97828091-cargo-clippy-all-targets-d-warnings
    claim: "On Linux at 97828091: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3129 passed, 0 failed, 2 ignored and integration 705 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI."
    result: passed
    revision: 97828091ad38ab5bdc7c4ab49f1d9554cfae8be3
    environment: Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 97828091, as a normal user with jq, through one-lane
    content: 2c8e78ca4b990e9ef05fe352e68d6be0504a860db9de22e3267d1f77e5762688
    fingerprint: sha256:4c9675140e3d4ef96b418c3c526f81b146cdd2f972925814080a8396c2ecd10d
    observed: 2026-11-20T04:12:05Z
    actor: claude-code (hc t5r)
```
