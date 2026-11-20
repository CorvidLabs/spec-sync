---
id: 2026-11-20-hooks-install-uninstall-and-status-name-the-path-and-reason
title: hooks install, uninstall and status name the path and reason when a…
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T16:42:17Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-hook-read-errors-and-linked-cache.md
  - .specsync/archive/changes/2026-11-20-name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash/
record: .chough/archive/sessions/2026-11-20-hook-read-errors-and-linked-cache.md
actor:
  - agent: claude-code
    terminal: t5r
---
# hooks install, uninstall and status name the path and reason when a…

hooks install, uninstall and status name the path and reason when a hook target's read is refused or fails (oversized, not UTF-8, not a regular file, a link on the way to the hooks folder), and rehash refuses a user-cache hashes.json that is a link or not a regular file, so the code matches REQ-hooks-007, REQ-hooks-009 and REQ-hash-cache-007 as approved (change 9, from the reviews on #1133, private, with…

Why: 0xLeif decided "Scope approval of name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash": "Yes, change 9 (Recommended)" (Claude Code question dialog with…

Verified: Change 9: the four red tests (an oversized and a non-UTF-8 CLAUDE.md, a linked hooks folder, a linked user-cache hashes.json) failed on 7527da3a and pass at 852ec92a; the two controls pass; full…

Read more: [the story](../../.chough/archive/sessions/2026-11-20-hook-read-errors-and-linked-cache.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-name-the-path-and-reason-for-a-refused-hook-read-and-refuse-a-linked-user-cache-hashes-json-in-rehash/)
