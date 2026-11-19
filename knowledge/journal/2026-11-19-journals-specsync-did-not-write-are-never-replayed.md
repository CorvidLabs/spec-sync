---
id: 2026-11-19-journals-specsync-did-not-write-are-never-replayed
title: Journals SpecSync did not write are never replayed
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-19
time: 2026-11-19T19:54:18Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-19-journal-replay-targets.md
  - .specsync/archive/changes/2026-11-19-never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files/
record: .chough/archive/sessions/2026-11-19-journal-replay-targets.md
actor:
  - agent: claude-code
    terminal: t5r
---
# Journals SpecSync did not write are never replayed

Every writing change command replayed .specsync/change-transaction.json at lock time into any file under the root, so a committed journal could rewrite .git/config and run a command through core.fsmonitor (5.0.0 to 6.3.0).

A journal Git tracks is now refused with the git rm fix; entries replay only into .specsync/ and spec files (under specs_dir only *.spec.md, requirements.md and history fragments, judged after backslashes become separators), never .git in any case; one bad entry refuses the whole journal; the 5.x bare-array form is refused. The same rule guards journal writes.

Verified: 5 of 6 journal unit tests and the CLI test failed first, and the amendment's 2 tests (a backslash entry, specs_dir .) failed on the first fix; all 8 and the CLI test pass, and the full verify lane is green on macOS and the test suites on Linux.

Read more: [the story](../../.chough/archive/sessions/2026-11-19-journal-replay-targets.md) · [the archived change](../../.specsync/archive/changes/2026-11-19-never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files/)
