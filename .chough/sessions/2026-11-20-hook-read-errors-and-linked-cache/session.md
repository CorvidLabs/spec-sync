---
kind: session
id: 2026-11-20-hook-read-errors-and-linked-cache
aliases: [hook-read-errors-and-linked-cache]
objective: "hooks install, uninstall and status name the path and reason when a hook target's read is refused or fails (oversized, not UTF-8, not a regular file, a link on the way to the hooks folder), and rehash refuses a user-cache hashes.json that is a link or not a regular file, so the code matches REQ-hooks-007, REQ-hooks-009 and REQ-hash-cache-007 as approved (change 9, from the reviews on #1133, private, with GHSA-7q5v-8jcc-rx54)"
intent_scope: ""
modules: [hooks, hash_cache]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
pr: ""
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-20-change-9-the-four-red-tests-an-oversized-and-a-non-utf-8
  - 2026-11-20-change-9-the-four-red-tests-an-oversized-and-a-non-utf-8-205d35
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
    holds: accept
    drives: review
    opened: 2026-11-20T16:31:01Z
run_consents: []
next: Continue accept
---
# hooks install, uninstall and status name the path and reason when a hook target's read is refused or fails (oversized, not UTF-8, not a regular file, a link on the way to the hooks folder), and rehash refuses a user-cache hashes.json that is a link or not a regular file, so the code matches REQ-hooks-007, REQ-hooks-009 and REQ-hash-cache-007 as approved (change 9, from the reviews on #1133, private, with GHSA-7q5v-8jcc-rx54)

## Goal

hooks install, uninstall and status name the path and reason when a hook target's read is refused or fails (oversized, not UTF-8, not a regular file, a link on the way to the hooks folder), and rehash refuses a user-cache hashes.json that is a link or not a regular file, so the code matches REQ-hooks-007, REQ-hooks-009 and REQ-hash-cache-007 as approved (change 9, from the reviews on #1133, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash": "Yes, change 9 (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 15:43Z, answering 'Security PR #1133: CI is all green, but corvid-agent requested changes, which blocks the merge. Its must-fix: an oversized or non-UTF-8 hook file is reported as "not installed" instead of naming the path and reason, which the approved REQ-hooks-007 requires. Also, Codex and corvid-agent both found that rehash silently replaces a user-cache hashes.json that is a link instead of refusing (nothing is written through the link, but the notice says rehash exits 1). Both are small, and both make the code match wording you already approved. Plan: fix both as change 9 with failing tests first, Mac and Linux green, about 1-2 hours; you take one quick look; then merge and RC. A third item (hooks in a linked worktree with no shared hooks folder) goes to 6.4.1. OK?'. t4c's brief: hooks install, uninstall and status propagate a refused or failed read of a hook target with the path and the reason (uninstall exits 1, status prints the reason); rehash refuses a user-cache hashes.json that is a link or not a regular file, exit 1, with check's wording. The pre-commit uninstall's linked-hooks-folder case (corvid-agent: 'worth folding in') is included as the same REQ-hooks-009 refusal. Relayed by t4c over hc to t5r; a relayed claim, not authenticated); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash as 0xLeif

## What was verified

- [Change 9: the four red tests (an oversized and a non-UTF-8 CLAUDE.md, a linked hooks folder, a linked user-cache hashes.json) failed on 7527da3a and pass at 852ec92a; the two controls pass; full suites green (3158 unit, 730 integration), clippy clean, check --strict 100%](evidence/2026-11-20-change-9-the-four-red-tests-an-oversized-and-a-non-utf-8.md)
- [Change 9: the four red tests (an oversized and a non-UTF-8 CLAUDE.md, a linked hooks folder, a linked user-cache hashes.json) failed on 7527da3a and pass at 852ec92a; the two controls pass; full suites green (3158 unit, 730 integration), clippy clean, check --strict 100%](evidence/2026-11-20-change-9-the-four-red-tests-an-oversized-and-a-non-utf-8-205d35.md)

## Open threads

## Timeline

- 2026-11-20T15:45:20Z · start · intent → intent · hooks install, uninstall and status name the path and reason when a hook target's read is refused or fails (oversized, not UTF-8, not a regular file, a link on the way to the hooks folder), and rehash refuses a user-cache hashes.json that is a link or not a regular file, so the code matches REQ-hooks-007, REQ-hooks-009 and REQ-hash-cache-007 as approved (change 9, from the reviews on #1133, private, with GHSA-7q5v-8jcc-rx54) ^t-fb91cd
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - by: claude-code (hc t5r)
- 2026-11-20T15:45:33Z · advance · intent → design ·  ^t-731209
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - by: claude-code (hc t5r)
- 2026-11-20T15:45:33Z · advance · design → plan ·  ^t-259716
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - by: claude-code (hc t5r)
- 2026-11-20T15:48:03Z · gate-opened · plan → plan · Scope approval of name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash ^t-5cd49a
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T15:49:27Z · direct-native-write · plan → plan · name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-0fc999
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - direct_write:
    - change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t5r)
- 2026-11-20T15:49:27Z · gate-closed · plan → plan · Scope approval of name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash ^t-62059c
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - relayed_by:
    - decided_by: 0xLeif
    - words: Yes, change 9 (Recommended)
    - where: Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 15:43Z, answering 'Security PR #1133: CI is all green, but corvid-agent requested changes, which blocks the merge. Its must-fix: an oversized or non-UTF-8 hook file is reported as "not installed" instead of naming the path and reason, which the approved REQ-hooks-007 requires. Also, Codex and corvid-agent both found that rehash silently replaces a user-cache hashes.json that is a link instead of refusing (nothing is written through the link, but the notice says rehash exits 1). Both are small, and both make the code match wording you already approved. Plan: fix both as change 9 with failing tests first, Mac and Linux green, about 1-2 hours; you take one quick look; then merge and RC. A third item (hooks in a linked worktree with no shared hooks folder) goes to 6.4.1. OK?'. t4c's brief: hooks install, uninstall and status propagate a refused or failed read of a hook target with the path and the reason (uninstall exits 1, status prints the reason); rehash refuses a user-cache hashes.json that is a link or not a regular file, exit 1, with check's wording. The pre-commit uninstall's linked-hooks-folder case (corvid-agent: 'worth folding in') is included as the same REQ-hooks-009 refusal. Relayed by t4c over hc to t5r; a relayed claim, not authenticated
    - recorded_by: claude-code-t5r
    - native: specsync change approve name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T15:49:42Z · advance · plan → build · native:approved ^t-7d3735
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - gate: native:approved
  - revision: 7527da3a7aea52436404766df845bbc8fb2f19b6
  - by: claude-code (hc t5r)
- 2026-11-20T16:30:45Z · advance · build → test · native:verifying ^t-a2589d
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - gate: native:verifying
  - revision: 266b979d35d07e69e938eb692338253e189a6d5c
  - by: claude-code (hc t5r)
- 2026-11-20T16:30:53Z · advance · test → accept · native:verifying ^t-5fbc05
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - gate: native:verifying
  - revision: 266b979d35d07e69e938eb692338253e189a6d5c
  - by: claude-code (hc t5r)
- 2026-11-20T16:31:01Z · gate-opened · accept → accept · Implementation review of name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash ^t-01acea
  - native_change: name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash
  - revision: 266b979d35d07e69e938eb692338253e189a6d5c
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
