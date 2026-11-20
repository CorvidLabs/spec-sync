---
kind: session
id: 2026-11-20-user-cache-only-state
aliases: [user-cache-only-state]
objective: SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules: [util, change, hooks, git_utils, cmd_check]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: keep-specsync-s-own-state-only-in-the-user-cache-folder
pr: ""
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-20-on-e3d508ac-with-only-change-7-s-tests-added-3fcb6de7-all-3
  - 2026-11-20-with-change-7-s-fix-at-97828091-fledge-lanes-run-verify-passes
  - 2026-11-20-on-linux-at-97828091-cargo-clippy-all-targets-d-warnings
bugs: [2026-11-20-state-an-archive-ships-in-its-git-directory-is-trusted]
blockers: []
waiting_on_human: []
run_consents: []
next: Continue build
---
# SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)

## Goal

SpecSync keeps its cache, journal and remote cache only in the user's cache folder, never in a Git directory an archive can ship; the enclosing-work-tree check reads paths literally; registry replay is spec-shaped; a hooks path outside both trees comes from the person's own config (R1-R5a, N2 and R6 of the change 6 re-check, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of keep-specsync-s-own-state-only-in-the-user-cache-folder": "A" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 03:27Z, answering t4c's question after sec-review2's re-check of change 6 (security/review2/recheck-change6.md): 'A (recommended): one more small change, about half a day. SpecSync keeps its cache only in your user cache folder, never in .git. That's simpler, one rule fewer, and closes the archive case and two related ones. It also fixes the one-line bug and the edge cases, then the reviewer does a quick re-check of just those. 6.4.0 moves to about Nov 25-26, and the security notice's claims stay true as written.' The one-line bug was R4 (a folder named * as a wildcard) and the edge cases R2, R5a and N2; t4c's brief lists items 1-5 (user cache folder only keyed by the canonical root; refuse a cache base inside the root or the confinement top; --literal-pathspecs and a clean environment in tracked_by_git; spec-shaped registry replay; a hooks path outside both trees only from global or system scope) and, as text and tests only, R6's small scan patterns and the documentation fixes, R5b documented; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve keep-specsync-s-own-state-only-in-the-user-cache-folder as 0xLeif

## What was verified

- [On e3d508ac with only change 7's tests added (3fcb6de7): all 3 new unit tests and 5 of 7 new integration tests fail (also REQ-git-utils-012, REQ-cmd-check-021). The state folder of a Git work tree is .git/specsync/<constant key>; a copy of a project with its .git replays the cache it ships (Skipped 1 unchanged spec, Cache: unpacked/.git/specsync/...); a valid journal at .git/specsync/<key>/change-transaction.json is replayed; a registry entry env = .env lets a journal delete .env; with XDG_CACHE_HOME inside the project the cache there is replayed; a project folder named * inside a home work tree that tracks Downloads/readme.txt is widened and the outer notes.md gets the CLAUDE.md block; a local core.hooksPath ../outside/made/hooks makes outside/made. The read scan finds 10 new unmarked opens (open_ambient_dir and .open(). The 2 that pass are the global-config hooks control and the write scan.](evidence/2026-11-20-on-e3d508ac-with-only-change-7-s-tests-added-3fcb6de7-all-3.md)
- [With change 7's fix at 97828091, fledge lanes run verify passes all 10 steps: unit 3125 passed, 0 failed, 2 ignored; integration 702 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 3 new unit tests and 7 new integration tests pass (also REQ-git-utils-012 and REQ-cmd-check-021); the state folder is in the user cache folder, a copy of a project with its .git validates every spec, a cache folder inside the project is refused, a folder named * stays confined, a registry entry naming .env opens no replay, a local core.hooksPath outside makes no folder, and the scans find nothing unmarked.](evidence/2026-11-20-with-change-7-s-fix-at-97828091-fledge-lanes-run-verify-passes.md)
- [On Linux at 97828091: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3129 passed, 0 failed, 2 ignored and integration 705 passed, 0 failed. Not run: x86_64, the Ubuntu image CI uses, hosted CI.](evidence/2026-11-20-on-linux-at-97828091-cargo-clippy-all-targets-d-warnings.md)

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
