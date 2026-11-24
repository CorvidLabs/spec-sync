---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: testing
---

# Testing

Failing first: the tests below are committed before the code, and the refusing ones fail there because
finalize, archive and ship-status archive or report whatever ledger the working tree holds.

- `src/change_tests/committed_approvals.rs`
  - Refused: one approval's actor rewritten in the working tree after review (finalize names the file,
    `HEAD` and the restore, and archives nothing; restoring lets it archive); that rewrite committed
    (names the rewriting commit); a dropped approval, a rewritten reopening and a dropped send-back,
    committed; a ledger never committed; a reopened change checked without a commit (names the commit,
    never the restore); a workspace deleted and created again under the same id; a committed version
    that drops a member an older SpecSync did not know; a rewrite on a branch merged in, refused at the
    merge commit; an accepted change whose working copy adds another approval or changes a committed
    one.
  - Still finalizes: a reopen; a send-back and fresh approval; a relay committed and then removed, and
    one restored, in an approval and in a reopening's `superseded_approval`; the two migrate-5.0 digest
    members added to a reopening; a top-level member this version does not know; a reopen out of the
    archive; an accepted change archived with the closing records `change accept` wrote; a finalization
    that stopped after acceptance, resumed; the usual flow.
  - Refused beside the nested relay: a committed change to another member of a reopening's embedded
    approval.
- `tests/integration/committed_approvals.rs`, through the binary: the working-tree rewrite refused by
  `change finalize`, `change ship` and `change ship --dry-run` with the same message;
  `change ship-status --json` lists it as a blocker with `ready_to_finalize` false, and true again once
  the file is restored; `change accept` then `change archive` archives with the uncommitted closing
  records.
- The existing suite: every test that finalizes or archives passes through the check.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-166` | `src/change_tests/committed_approvals.rs`: each refusal and its way out, each legitimate writer, the accepted-change exception, the merged-in rewrite; `tests/integration/committed_approvals.rs`: finalize, ship, accept then archive |
| `REQ-cmd-change-committed-ledger-001` | `tests/integration/committed_approvals.rs`: the ship-status blocker, `ready_to_finalize` false then true, and the `ship --dry-run` refusal with the same message |
