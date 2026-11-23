---
id: 2026-11-19-hash-cache-never-read-or-written-through-a-link
title: Hash cache never read or written through a link
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T14:06:33Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-hash-cache-symlink-write.md
  - .specsync/archive/changes/2026-11-19-never-read-or-write-the-hash-cache-through-a-symbolic-link/
record: .chough/archive/sessions/2026-11-19-hash-cache-symlink-write.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Hash cache never read or written through a link

specsync check and rehash wrote .specsync/hashes.json with a write that follows a symbolic link, so a repository that committed that path as a link could make a check in a clone overwrite a file outside it (GHSA-7q5v-8jcc-rx54, 3.3.0 to 6.3.0).

The cache is now read and written through a no-follow handle on .specsync and saved by rename; any link is refused. check warns once and keeps its verdict and exit code, rehash exits 1.

Verified: 5 unit and 2 integration tests failed first and pass; full macOS verify lane green; Linux arm64 container green. Next: the project file change on the same private PR, then the advisory.

Read more: [the story](../../.chough/archive/sessions/2026-11-19-hash-cache-symlink-write.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-never-read-or-write-the-hash-cache-through-a-symbolic-link/)
