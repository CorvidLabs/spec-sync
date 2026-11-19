---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: tasks
---

# Tasks

- [ ] Failing unit tests in `src/change_tests/committed_approvals.rs`: the working-tree rewrite, the committed rewrite of each list, the never-committed ledger, the uncommitted append, the delete-and-recreate, the older binary's dropped member, the merged-in rewrite, and the accepted-change exception and its refusals
- [ ] Unit tests that each legitimate writer still finalizes: reopen, send-back and approve, repair-relays in an approval and in a reopening, the migrate-5.0 digest members, an unknown ledger member, and a reopen out of the archive
- [ ] Failing integration tests through the binary: finalize, ship, `ship --dry-run`, ship-status's blocker and `ready_to_finalize`, and accept then archive
- [ ] The check in `src/change/historical_definition_digest_matches.rs`: the extends rule on the relay-blind projection, and the refusal for each way out
- [ ] Ask the check in finalize, archive and ship-status
- [ ] Existing tests that finalize or archive an uncommitted ledger gain a commit; the tamper test expects the new message
- [ ] The `files:` entry for the new test file in the `change` spec
- [ ] Docs: `workflow.md`, `cli.md`, and `CHANGELOG.md` with the 6.3.0 note
- [ ] `fledge lanes run verify` and the pre-push gate
- [ ] After #1093 merges and this branch is updated against main: assert that `change ship-status <id>` exits 1 on the ledger blocker (REQ-cmd-change-041's criterion; exit 1 on a blocker is #654's REQ-cmd-change-039)

Human gates and Chough's own records are not tasks here: Leif's scope approval and implementation review
are recorded through Chough when he gives them.
