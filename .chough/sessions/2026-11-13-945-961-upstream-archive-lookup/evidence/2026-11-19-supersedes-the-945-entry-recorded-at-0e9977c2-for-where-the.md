---
kind: evidence
id: 2026-11-19-supersedes-the-945-entry-recorded-at-0e9977c2-for-where-the
aliases: []
claim: "Supersedes the #945 entry recorded at 0e9977c2 for where the real-flow test lives. change audit --strict refused tests/integration/archived_upstream.rs (not among #945's declared paths), so the pushed 77723e4f fails the strict audit. The same real flow (finalize and archive on main, branch, stale the evidence, reopen_change, check, upstream rename, record_scoped_review) now runs as change::tests::a_reopened_change_reviews_after_the_remote_default_renames_its_archive in src/change_tests/upstream_archive.rs with a bare origin and the no-rename control. It fails on f740d0a2's lookup (Archived at the rename commit) and passes now. The integration file is back to main's bytes. fledge lanes run verify at a783d291 with GIT_CONFIG_GLOBAL=/dev/null: 10/10 steps, 3080 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage."
result: passed
evidence_kind: product-test
revision: a783d291336f4ac9538a3c0706d20685c70efd0c
content: 3af1d0333fe8b5b53cec8d1c6b38bdb0e39ade79c1008a6b41aeb066eff772a7
environment: local macOS 26, rust 1.89.0, branch fix/upstream-archive-945-961-965 at a783d291
criteria: [SHIP-3.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T08:43:44Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Supersedes the #945 entry recorded at 0e9977c2 for where the real-flow test lives. change audit --strict refused tests/integration/archived_upstream.rs (not among #945's declared paths), so the pushed 77723e4f fails the strict audit. The same real flow (finalize and archive on main, branch, stale the evidence, reopen_change, check, upstream rename, record_scoped_review) now runs as change::tests::a_reopened_change_reviews_after_the_remote_default_renames_its_archive in src/change_tests/upstream_archive.rs with a bare origin and the no-rename control. It fails on f740d0a2's lookup (Archived at the rename commit) and passes now. The integration file is back to main's bytes. fledge lanes run verify at a783d291 with GIT_CONFIG_GLOBAL=/dev/null: 10/10 steps, 3080 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage.**

Recorded as passed in local macOS 26, rust 1.89.0, branch fix/upstream-archive-945-961-965 at a783d291 at revision a783d291336f, observed 2026-11-19T08:43:44Z.
