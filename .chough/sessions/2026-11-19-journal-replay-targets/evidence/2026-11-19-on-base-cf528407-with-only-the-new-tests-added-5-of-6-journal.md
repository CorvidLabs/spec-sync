---
kind: evidence
id: 2026-11-19-on-base-cf528407-with-only-the-new-tests-added-5-of-6-journal
aliases: []
claim: "On base cf528407 with only the new tests added: 5 of 6 journal unit tests and the change new CLI test fail. A journal entry rewrote .git/config, .GIT/config, a hook, or deleted .git/HEAD; a journal Git tracks was replayed; a journal with one entry for Makefile was replayed whole; a bare-array journal was replayed; a file beside a registry-mapped spec was written; and change new with a committed journal rewrote .git/config."
result: failed
evidence_kind: product-test
method: "cargo test --locked --bin specsync -- (journal tests: 3 passed, 5 failed); cargo test --locked --test integration -- change_new_never_replays_a_committed_journal_into_git (1 failed); rustc 1.89.0, isolated git config; each fixture its own git init"
revision: cf5284076db2b692d8e8df8953eb6525d5952690
content: 7841851ee75780c33d475f437daf93649c4ce5fb4627cda3b233e9e47fcfa701
environment: local-macos-arm64
criteria: [SETUP-8]
requirements: []
contracts: []
limitations:
  - "The passing control a_real_interrupted_write_is_still_recovered is meant to pass on base. The fsmonitor value planted in the fixture's .git/config never ran: no git command ran in that fixture afterwards."
observed: 2026-11-19T16:02:33Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On base cf528407 with only the new tests added: 5 of 6 journal unit tests and the change new CLI test fail. A journal entry rewrote .git/config, .GIT/config, a hook, or deleted .git/HEAD; a journal Git tracks was replayed; a journal with one entry for Makefile was replayed whole; a bare-array journal was replayed; a file beside a registry-mapped spec was written; and change new with a committed journal rewrote .git/config.**

Recorded as failed in local-macos-arm64 at revision cf5284076db2, observed 2026-11-19T16:02:33Z.

## Key failing lines

- called Result::unwrap_err() on an Ok value (the replay accepted the journal); a committed journal rewrote .git/config
