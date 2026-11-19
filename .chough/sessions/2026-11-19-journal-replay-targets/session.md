---
kind: session
id: 2026-11-19-journal-replay-targets
aliases: [journal-replay-targets]
objective: A transaction journal SpecSync did not write itself is never replayed, and no replay touches Git's files or anything the change workflow does not write (F1, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules: [change]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
pr: ""
criteria: [SETUP-8]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-on-base-cf528407-with-only-the-new-tests-added-5-of-6-journal
  - 2026-11-19-with-the-fix-the-6-journal-unit-tests-and-the-change-new-cli
  - 2026-11-19-on-change-4-s-tip-75dddd76-two-new-tests-fail-the-path-rule
  - 2026-11-19-with-the-amendment-the-rule-judges-the-normalized-path-under
bugs: [2026-11-19-a-committed-transaction-journal-is-replayed-into-git-s-files]
blockers: []
waiting_on_human:
  - gate: Implementation review of never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
    holds: accept
    drives: review
    opened: 2026-11-19T16:36:20Z
run_consents: []
next: Continue build
title: Journals SpecSync did not write are never replayed
summary:
  - Every writing change command replayed .specsync/change-transaction.json at lock time into any file under the root, so a committed journal could rewrite .git/config and run a command through core.fsmonitor (5.0.0 to 6.3.0).
  - A journal Git tracks is now refused with the git rm fix; entries replay only into .specsync/, specs_dir and registry-mapped specs, never .git in any case; one bad entry refuses the whole journal; the 5.x bare-array form is refused. The same rule guards journal writes.
  - "Verified: 5 unit tests and the CLI test failed first and pass; full macOS verify lane green."
---
# A transaction journal SpecSync did not write itself is never replayed, and no replay touches Git's files or anything the change workflow does not write (F1, private, with GHSA-7q5v-8jcc-rx54)

## Goal

A transaction journal SpecSync did not write itself is never replayed, and no replay touches Git's files or anything the change workflow does not write (F1, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Intent approval of proposed criterion SETUP-8": "yes" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:55Z, after t4c summed up the change 4 scope details: affected 5.0.0 to 6.3.0; SETUP-8 as written; a tracked journal refuses the command with the git rm fix; replay only into .specsync/, specs_dir and registry-mapped spec files with their requirements.md and history/, .git refused case-insensitively with trailing dots and spaces stripped; one entry outside refuses the whole journal; the legacy form refused with the existing invalid-journal message and recovery; journal-into-.git and a keyed digest left out. His earlier yes at about 15:49Z approved the three points. Relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated
- 0xLeif decided "Scope approval of never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files": "yes" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:55Z, after t4c summed up the change 4 scope details: affected 5.0.0 to 6.3.0; SETUP-8 as written; a tracked journal refuses the command with the git rm fix; replay only into .specsync/, specs_dir and registry-mapped spec files with their requirements.md and history/, .git refused case-insensitively with trailing dots and spaces stripped; one entry outside refuses the whole journal; the legacy form refused with the existing invalid-journal message and recovery; journal-into-.git and a keyed digest left out. His earlier yes at about 15:49Z approved the three points. Relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files as 0xLeif

## What was verified

- [On base cf528407 with only the new tests added: 5 of 6 journal unit tests and the change new CLI test fail. A journal entry rewrote .git/config, .GIT/config, a hook, or deleted .git/HEAD; a journal Git tracks was replayed; a journal with one entry for Makefile was replayed whole; a bare-array journal was replayed; a file beside a registry-mapped spec was written; and change new with a committed journal rewrote .git/config.](evidence/2026-11-19-on-base-cf528407-with-only-the-new-tests-added-5-of-6-journal.md)
- [With the fix, the 6 journal unit tests and the change new CLI test pass, and the full verify lane passes: unit 3104 passed, 0 failed, 2 ignored; integration 666 passed, 0 failed; fmt, clippy, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts.](evidence/2026-11-19-with-the-fix-the-6-journal-unit-tests-and-the-change-new-cli.md)
- [On change 4's tip 75dddd76: two new tests fail. The path rule checked the raw entry while the write turns backslashes into separators, so a journal entry .git\config under specs_dir . and specs/.git\config under specs were replayed; and specs_dir . admitted a Makefile entry.](evidence/2026-11-19-on-change-4-s-tip-75dddd76-two-new-tests-fail-the-path-rule.md)
- [With the amendment (the rule judges the normalized path; under specs_dir only spec files, requirements.md and history fragments), all 8 journal unit tests and the change new CLI test pass, and the full verify lane passes: unit 3106 passed, 0 failed, 2 ignored; integration 666 passed, 0 failed; fmt, clippy, the release build, the strict spec check, the gate scripts.](evidence/2026-11-19-with-the-amendment-the-rule-judges-the-normalized-path-under.md)

## Open threads

## Timeline

- 2026-11-19T15:59:32Z · start · intent → intent · A transaction journal SpecSync did not write itself is never replayed, and no replay touches Git's files or anything the change workflow does not write (F1, private, with GHSA-7q5v-8jcc-rx54) ^t-3cad18
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - by: claude-code (hc t5r)
- 2026-11-19T16:00:56Z · gate-opened · intent → intent · Intent approval of proposed criterion SETUP-8 ^t-6d898b
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - holds: intent
  - drives: none
  - by: claude-code (hc t5r)
- 2026-11-19T16:00:57Z · gate-opened · intent → intent · Scope approval of never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files ^t-308f01
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-19T16:00:57Z · gate-closed · intent → intent · Intent approval of proposed criterion SETUP-8 ^t-f4e5a1
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:55Z, after t4c summed up the change 4 scope details: affected 5.0.0 to 6.3.0; SETUP-8 as written; a tracked journal refuses the command with the git rm fix; replay only into .specsync/, specs_dir and registry-mapped spec files with their requirements.md and history/, .git refused case-insensitively with trailing dots and spaces stripped; one entry outside refuses the whole journal; the legacy form refused with the existing invalid-journal message and recovery; journal-into-.git and a keyed digest left out. His earlier yes at about 15:49Z approved the three points. Relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
  - by: claude-code (hc t5r)
- 2026-11-19T16:00:57Z · gate-closed · intent → intent · Scope approval of never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files ^t-ef422b
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:55Z, after t4c summed up the change 4 scope details: affected 5.0.0 to 6.3.0; SETUP-8 as written; a tracked journal refuses the command with the git rm fix; replay only into .specsync/, specs_dir and registry-mapped spec files with their requirements.md and history/, .git refused case-insensitively with trailing dots and spaces stripped; one entry outside refuses the whole journal; the legacy form refused with the existing invalid-journal message and recovery; journal-into-.git and a keyed digest left out. His earlier yes at about 15:49Z approved the three points. Relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T16:01:06Z · advance · intent → design ·  ^t-234d8f
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - by: claude-code (hc t5r)
- 2026-11-19T16:01:06Z · skip · design → plan · No visuals or interactions ^t-781714
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - by: claude-code (hc t5r)
- 2026-11-19T16:01:08Z · advance · plan → build · native:approved ^t-a2ed70
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - gate: native:approved
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - by: claude-code (hc t5r)
- 2026-11-19T16:36:15Z · advance · build → test · native:verifying ^t-d90e22
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - gate: native:verifying
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - by: claude-code (hc t5r)
- 2026-11-19T16:36:19Z · advance · test → accept · native:verifying ^t-138167
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - gate: native:verifying
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - by: claude-code (hc t5r)
- 2026-11-19T16:36:19Z · gate-opened · accept → accept · Implementation review of never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files ^t-98081a
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - revision: cf5284076db2b692d8e8df8953eb6525d5952690
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
- 2026-11-19T16:39:30Z · back · accept → build · t4c's read found that the path rule checked the raw entry while the write normalizes backslashes, so '.git\config' passed, and specs_dir '.' admitted every path; amending inside the approved rule ^t-2790c0
  - native_change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
  - gate: native:verifying
  - revision: 75dddd7695acbbb2106feeb9889d315cd64297f8
  - refresh: build: implementation against the current definition, test: rerun affected cases; earlier observations may be stale, accept: native verification and the human implementation review, checkpoint references: 0 case(s) and 2 observation(s) to re-check with `chough status`
  - by: claude-code (hc t5r)
