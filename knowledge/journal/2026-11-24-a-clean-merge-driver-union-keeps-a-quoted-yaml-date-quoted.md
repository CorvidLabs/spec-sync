---
id: 2026-11-24-a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
title: A clean merge-driver union keeps a quoted YAML date quoted
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-24
time: 2026-11-24T00:12:31Z
kind: shipped
links:
  - https://github.com/CorvidLabs/spec-sync/pull/1151
  - .chough/archive/sessions/2026-11-19-1012-merge-keeps-quoted-timestamps.md
  - .specsync/archive/changes/2026-11-23-a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted/
record: .chough/archive/sessions/2026-11-19-1012-merge-keeps-quoted-timestamps.md
actor:
  - terminal: t14v
---
# A clean merge-driver union keeps a quoted YAML date quoted

A clean merge-driver union keeps a quoted YAML date quoted (#1012)

Why: authorization is Leif's own decision ('#1012 quoted dates', coordinator chat 2026-11-13T03:57Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form on 2026-11-13 because that is when he gave it; this session only refreshes the change onto main

Verified: At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes merge_driver::tests::quoted_yaml_timestamps_stay_strings_after_a_union.

Read more: [the pull request](https://github.com/CorvidLabs/spec-sync/pull/1151) · [the story](../../.chough/archive/sessions/2026-11-19-1012-merge-keeps-quoted-timestamps.md) · [the archived change](../../.specsync/archive/changes/2026-11-23-a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted/)
