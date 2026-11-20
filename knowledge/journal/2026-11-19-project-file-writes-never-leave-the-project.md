---
id: 2026-11-19-project-file-writes-never-leave-the-project
title: Project file writes never leave the project
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T14:09:03Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-symlink-sweep-writes.md
  - .specsync/archive/changes/2026-11-19-never-write-a-project-file-through-a-symbolic-link/
record: .chough/archive/sessions/2026-11-19-symlink-sweep-writes.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Project file writes never leave the project

Install, scaffold, import, lifecycle-free spec creation, migrate, MCP lock and remote-cache commands wrote fixed project paths with writes that follow symbolic links, so a cloned repository could make them change or create files outside it. Found by the private sweep after GHSA-7q5v-8jcc-rx54.

util now reads and writes through a cap-std handle on the project root: a link inside the project is followed, a link out, a dangling link or a non-regular target is refused naming the path; migrate refuses before its first step; the remote cache refuses any link.

Verified: 15 integration and 3 unit tests failed first and pass; full macOS verify lane green; Linux arm64 container green. Next: advisory text, then publishing and release on Leif's yes.

Read more: [the story](../../.chough/archive/sessions/2026-11-19-symlink-sweep-writes.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-never-write-a-project-file-through-a-symbolic-link/)
