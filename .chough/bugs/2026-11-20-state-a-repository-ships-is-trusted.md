---
kind: bug
id: 2026-11-20-state-a-repository-ships-is-trusted
aliases: []
title: State a repository ships is trusted
status: open
classification: unclassified
scope: in-scope
impact: A false PASS from a plain check on macOS and in unpacked trees; untracked user files rewritten or deleted by a replayed journal; a forged resolve --verify; a file created outside the clone and made executable; a link to a user's own file followed. Private, with GHSA-7q5v-8jcc-rx54
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-006, REQ-util-007, REQ-change-162, REQ-hooks-009]
cases: []
evidence: []
receipts: []
environment: local-macos-arm64 (APFS), rustc 1.89.0
revision: b2c70e1fe315e15d2305563920bb6cba9ecb4550
found_by: claude-code (hc t5r) (manual report)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-20T01:55:50Z
updated: 2026-11-20T01:55:50Z
---
# State a repository ships is trusted

**What happened:** A forged .specsync/hashes.json passes a failing spec in a plain check when committed as .specsync/Hashes.json on a case-insensitive disk, in an unpacked tree, or where git fails; a planted transaction journal is replayed; a committed .specsync-cache entry stands in for the remote spec; hooks install --precommit follows links on a core.hooksPath inside the work tree; an unpacked project inside another work tree is confined to the outer tree; the scans miss production code in tests.rs and several APIs

Actual: On a2a50824 with change 6's tests (b2c70e1f): 5 of 5 new unit tests and 8 of 13 new integration tests fail, and the scans list 12 reads, 7 writes and 5 Git calls

**Steps:** Trigger: Run specsync check, change new or another lifecycle command, resolve --verify, or hooks install --precommit in a clone or unpacked tree that ships these files or links

**Expected:** Nothing a repository ships is read as SpecSync's own state, and no link on a hooks path or in an unrelated enclosing work tree is followed

**Why it matters:** A false PASS from a plain check on macOS and in unpacked trees; untracked user files rewritten or deleted by a replayed journal; a forged resolve --verify; a file created outside the clone and made executable; a link to a user's own file followed. Private, with GHSA-7q5v-8jcc-rx54

**Fix:** Not fixed yet.

## Log

- 2026-11-20T01:55:50Z · Reported (manual report) · by claude-code (hc t5r)
