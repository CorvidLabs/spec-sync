---
id: 2026-11-19-change-3-s-call-sites-covered
title: Change 3's call sites covered
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T15:58:50Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-cover-root-call-sites.md
  - .specsync/archive/changes/2026-11-19-cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads/
record: .chough/archive/sessions/2026-11-19-cover-root-call-sites.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Change 3's call sites covered

After change 3 was archived, the change audit named six call-site files that change 3's commit edited but its paths did not declare. This coverage-only change declares them; no code or contract changes.

Verified: the audit passes with it; the full suites already ran on this code with change 3.

Read more: [the story](../../.chough/archive/sessions/2026-11-19-cover-root-call-sites.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads/)
