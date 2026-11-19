---
kind: bug
id: 2026-11-19-a-committed-transaction-journal-is-replayed-into-git-s-files
aliases: []
title: A committed transaction journal is replayed into Git's files
status: verified
classification: product-defect
scope: blocking
impact: A repository can run a command of its choosing through core.fsmonitor or a hook when someone starts the change workflow in a clone; 5.0.0 through 6.3.0
criteria: [SETUP-8]
requirements: []
cases: []
evidence:
  - 2026-11-19-journal-replay-targets/2026-11-19-on-base-cf528407-with-only-the-new-tests-added-5-of-6-journal
  - 2026-11-19-journal-replay-targets/2026-11-19-with-the-fix-the-6-journal-unit-tests-and-the-change-new-cli
  - 2026-11-19-journal-replay-targets/2026-11-19-on-change-4-s-tip-75dddd76-two-new-tests-fail-the-path-rule
  - 2026-11-19-journal-replay-targets/2026-11-19-with-the-amendment-the-rule-judges-the-normalized-path-under
receipts: []
fixture: src/change_tests/journal_replay_targets.rs and tests/integration/project_reads.rs; each fixture its own git init, nothing outside it
environment: local-macos-arm64
revision: cf5284076db2b692d8e8df8953eb6525d5952690
found_by: claude-code (hc t5r) (independent review of the private advisory branch (F1))
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-19T16:02:34Z
updated: 2026-11-19T17:00:34Z
---
# A committed transaction journal is replayed into Git's files

**What happened:** A writing change command rewrites .git/config, replaces a hook or deletes .git/HEAD before it starts

Actual: The lock replays every entry that passes safe_project_path, which allows any file under the root including .git/config; the bare-array form has no envelope and the durable digest is unkeyed

**Steps:** Trigger: Any writing specsync change command, such as change new, in a repository that commits .specsync/change-transaction.json
1. Commit .specsync/change-transaction.json with an entry for .git/config that sets core.fsmonitor
2. Clone the repository and run specsync change new
3. .git/config now holds the planted setting, and the next git status runs it

**Expected:** A journal Git tracks is never replayed; a replay writes only files the change workflow writes, never under .git; one bad entry refuses the whole journal

**Why it matters:** A repository can run a command of its choosing through core.fsmonitor or a hook when someone starts the change workflow in a clone; 5.0.0 through 6.3.0

**Fix:** Verified: 2026-11-19-journal-replay-targets/2026-11-19-with-the-fix-the-6-journal-unit-tests-and-the-change-new-cli passed at cf5284076db2b692d8e8df8953eb6525d5952690.

## Log

- 2026-11-19T16:02:34Z · Reported (independent review of the private advisory branch (F1)); evidence: 2026-11-19-journal-replay-targets/2026-11-19-on-base-cf528407-with-only-the-new-tests-added-5-of-6-journal · by claude-code (hc t5r)
- 2026-11-19T16:02:34Z · Added despite similar reports: This is about replaying a committed journal; the similar bugs are about links · by claude-code (hc t5r)
- 2026-11-19T16:35:49Z · status: open -> verified; evidence: 2026-11-19-journal-replay-targets/2026-11-19-with-the-fix-the-6-journal-unit-tests-and-the-change-new-cli. Fixed by the native change never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files: a journal Git tracks is refused, entries are replayed only into files the workflow writes and never under .git, one refused entry refuses the whole journal, and the bare-array form is refused. The tests that failed on cf528407 pass. · by claude-code (hc t5r)
- 2026-11-19T16:40:03Z · status: verified -> open; evidence: 2026-11-19-journal-replay-targets/2026-11-19-on-change-4-s-tip-75dddd76-two-new-tests-fail-the-path-rule. Reopened: the first fix checked the raw entry, so a backslash-separated .git entry passed, and specs_dir . widened the rule to every path. Amending change 4 before its review. · by claude-code (hc t5r)
- 2026-11-19T17:00:34Z · status: open -> verified; evidence: 2026-11-19-journal-replay-targets/2026-11-19-with-the-amendment-the-rule-judges-the-normalized-path-under. Amended: the rule now judges the normalized path and admits only spec-shaped files under specs_dir; the two tests that failed on 75dddd76 pass. · by claude-code (hc t5r)
