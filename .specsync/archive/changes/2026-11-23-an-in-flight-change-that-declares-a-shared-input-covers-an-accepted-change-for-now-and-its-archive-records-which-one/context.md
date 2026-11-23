---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: context
---

# Context

An accepted change goes stale the moment a later change edits one of its signed inputs, because only an accepted or archived successor covers the edit. While the later change is in flight, every accepted change that signed the same file fails `change audit --strict` at once; a team reopened six changes four times, about 20 minutes of re-verification each round (#657). When two accepted changes block each other's archive over one stale input, neither refusal says so (#688).

Leif's choices: for #657 on 2026-11-13 at about 04:04Z, "It may archive (Recommended)", so the archive records which change covered the shared file; for #688 on 2026-11-18, one sentence in each of the two archive refusals, built in the #657 PR. He approved this definition as written on 2026-11-19 at about 14:17Z ("Approve as written", relayed by hc t4c through ss-lead hc t58), with SHIP-13 and .a to .d as the agent's draft.

The unfinished branch `feat/657-provisional-coverage` (failing test `5063cf7d`, fix `195bca2d`, base bb7682a7) is the starting point; it carries no delta and nothing reads its `coverage.json`.
