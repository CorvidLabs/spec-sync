---
id: 2026-11-19-at-ship-specsync-writes-specs-module-history-change-id-md
title: At ship SpecSync writes specs/<module>/history/<change-id>.md…
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T01:46:35Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-1084-history-fragments.md
  - .specsync/archive/changes/2026-11-19-record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump/
record: .chough/archive/sessions/2026-11-19-1084-history-fragments.md
actor:
  - agent: claude-code
    terminal: t4t
---
# At ship SpecSync writes specs/\<module\>/history/\<change-id\>.md…

At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

Why: Authorization: Leif approved spec-sync #1084 on 2026-11-18, written on the issue as 'Leif's decision (2026-11-18): approved', relayed to this agent by hc t4c as his scope approval; scope is the issue's three bullets; limits: no release, tag or publish, no merge without his relayed yes

Verified: Covers REQ-change-159, -160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037 (the last two are not linked: chough 0.6.0 looks them up under specs/hash-cache and specs/cmd-change).

Read more: [the story](../../.chough/archive/sessions/2026-11-19-1084-history-fragments.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump/)
