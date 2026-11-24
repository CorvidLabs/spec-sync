---
id: 2026-11-19-project-file-reads-are-confined-and-bounded
title: Project file reads are confined and bounded
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T15:48:22Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-bounded-project-reads.md
  - .specsync/archive/changes/2026-11-19-never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound/
record: .chough/archive/sessions/2026-11-19-bounded-project-reads.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Project file reads are confined and bounded

SpecSync read about 200 project files with reads that follow links, block on FIFOs and have no bound, so a committed link from .specsync/sdd.json to /dev/zero exhausted memory in change status, which Chough runs on every next and status.

Every in-scope read now goes through util's bounded project read: confined to the root, non-blocking, type-checked on the handle, at most the bound plus one byte (4, 8 and 64 MiB); a source scan test keeps it that way.

Verified: 11 of 17 integration tests and the scan failed first and pass; full macOS verify lane green; Linux arm64 container green. Next: the advisory text, then publishing and release on Leif's yes.

Read more: [the story](../../.chough/archive/sessions/2026-11-19-bounded-project-reads.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound/)
