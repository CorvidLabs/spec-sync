---
id: 2026-11-25-check-reports-a-broken-lesson-file-on-a-cached-run
title: check reports a broken lesson file on a cached run
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-25
time: 2026-11-25T02:23:54Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-24-1163-lesson-file-follow-ups.md
  - .specsync/archive/changes/2026-11-25-check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file/
record: .chough/archive/sessions/2026-11-24-1163-lesson-file-follow-ups.md
actor:
  - agent: claude-code
    terminal: t14v
---
# check reports a broken lesson file on a cached run

Three review findings on #1163's lesson-file warning, each reproduced on the CLI: a cached check missed it, the file was named from the spec file rather than the module, and a directory of that name was skipped.

Read more: [the story](../../.chough/archive/sessions/2026-11-24-1163-lesson-file-follow-ups.md) · [the archived change](../../.specsync/archive/changes/2026-11-25-check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file/)
