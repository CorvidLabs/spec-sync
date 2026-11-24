---
id: 2026-11-24-check-fix-keeps-stdout-one-json-document-in-machine-formats
title: check --fix keeps stdout one JSON document in machine formats
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-24
time: 2026-11-24T00:12:23Z
kind: shipped
links:
  - https://github.com/CorvidLabs/spec-sync/pull/1151
  - .chough/archive/sessions/2026-11-13-475-check-fix-json-stdout.md
  - .specsync/archive/changes/2026-11-23-check-fix-keeps-stdout-one-json-document-in-machine-formats/
record: .chough/archive/sessions/2026-11-13-475-check-fix-json-stdout.md
actor:
  - terminal: t14v
---
# check --fix keeps stdout one JSON document in machine formats

check --fix keeps stdout one JSON document in machine formats, with progress on stderr (#475)

Why: authorization is Leif's own scope approval ('#475 clean JSON', coordinator chat 2026-11-13T04:03Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form

Verified: At f8b341c5 (#432, #475 and #1012 merged onto main 3bebe8b0, each change re-checked there), fledge lanes run verify passed 10/10 steps in 12m14s, run alone: fmt, lint and check-types, cargo test (unit 3078 passed, 2 ignored; integration 629 passed), release build, strict spec check (62 specs) and the CI script tests. The lane's processes ran with a neutral global git config (#1092). It includes fix::fix_with_json_output and fix::fix_dry_run_with_json_output_is_one_json_document.

Read more: [the pull request](https://github.com/CorvidLabs/spec-sync/pull/1151) · [the story](../../.chough/archive/sessions/2026-11-13-475-check-fix-json-stdout.md) · [the archived change](../../.specsync/archive/changes/2026-11-23-check-fix-keeps-stdout-one-json-document-in-machine-formats/)
