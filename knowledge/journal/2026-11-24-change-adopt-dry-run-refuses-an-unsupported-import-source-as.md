---
id: 2026-11-24-change-adopt-dry-run-refuses-an-unsupported-import-source-as
title: change adopt --dry-run refuses an unsupported import source as…
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-24
time: 2026-11-24T00:12:27Z
kind: shipped
links:
  - https://github.com/CorvidLabs/spec-sync/pull/1151
  - .chough/archive/sessions/2026-11-13-432-adopt-dry-run-validates-source.md
  - .specsync/archive/changes/2026-11-23-change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does/
record: .chough/archive/sessions/2026-11-13-432-adopt-dry-run-validates-source.md
actor:
  - terminal: t14v
---
# change adopt --dry-run refuses an unsupported import source as…

change adopt --dry-run refuses an unsupported import source as adoption does (#432)

Why: authorization is Leif's own scope approval ('#432 adopt dry run', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form

Verified: At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does.

Read more: [the pull request](https://github.com/CorvidLabs/spec-sync/pull/1151) · [the story](../../.chough/archive/sessions/2026-11-13-432-adopt-dry-run-validates-source.md) · [the archived change](../../.specsync/archive/changes/2026-11-23-change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does/)
