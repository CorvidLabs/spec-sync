---
id: 2026-11-20-cover-as-a-second-no-spec-change-change-in-the-6-4-1-pr
title: Cover, as a second no-spec-change change in the 6.4.1 PR
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T22:52:58Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-6-4-1-covering-paths.md
  - .specsync/archive/changes/2026-11-20-route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths/
record: .chough/archive/sessions/2026-11-20-6-4-1-covering-paths.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Cover, as a second no-spec-change change in the 6.4.1 PR

Cover, as a second no-spec-change change in the 6.4.1 PR, the 17 files the hardening change's approved W1, X1 and S3 fixes touched outside its path list (Leif: 'Yes, two changes (Recommended)')

Why: 0xLeif decided "Scope approval of route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths": "Yes, two changes (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 21:52Z, answering 'SpecSync 6.4.1: you chose "one change, one review". The fix routes every folder listing through the safe helper, which touched 17 files outside its approved list, and SpecSync can't add them as files only: each would need new requirement wording.

Read more: [the story](../../.chough/archive/sessions/2026-11-20-6-4-1-covering-paths.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths/)
