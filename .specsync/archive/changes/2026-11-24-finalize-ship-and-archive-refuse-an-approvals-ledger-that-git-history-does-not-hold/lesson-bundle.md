# Lesson bundle: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Finalize, ship and archive refuse an approvals ledger that Git history does not hold
- **Kind**: Feature
- **Specs**: change, cmd_change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/committed_approvals.rs, src/change_tests/approve_rejects_a_declared_path_owned_by_an_undeclared_module.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs, src/change_tests/recorded_review_recovery.rs, src/commands/change.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, site/src/content/docs/, CHANGELOG.md
- **Acceptance**: `change finalize`, `change ship` and `change archive` refuse, writing nothing, a change whose `approvals.json` is not committed at `HEAD` or differs from the committed version, naming the file and the way out: the commit when the working copy only adds records, the restore when it changes a committed one.
- **Acceptance**: They refuse when a committed version of the ledger rewrites or drops an earlier approval, reopening or send-back, naming the commit that did it, the commit before it and the record; that covers a ledger deleted and created again under the same id, and a record an older SpecSync rewrote without a member it did not know. A whole workspace deleted and created again is refused before that, by the workflow-version history.
- **Acceptance**: Every rewrite the tool itself makes still finalizes: a reopen, a send-back and fresh approval, a relay restored or removed in an approval, as `change repair-relays` does, the reopening members `specsync migrate 5.0` backfills, and a ledger member this version does not know. A relay restored or removed in the approval a reopening embeds still finalizes too.
- **Acceptance**: An accepted change archives with the closing records its accepting step wrote and has not committed, and no other uncommitted approval.
- **Acceptance**: `change ship-status` lists the refusal as a blocker and does not report the change ready, and `change ship --dry-run` refuses with it.
- **Acceptance**: The usual flow, approve, `check --commit`, review and finalize, archives as before.

## Evidence

- Verification commit: `f28652b32ab5eb45517012aaac1d0ebe80b52680`
- Base commit: `cf28b4c5052afb72ce01454a353d2e99c83b3222`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #428. `change finalize` archives whatever `approvals.json` the working tree holds. An approval
edited after it was committed, or one that was never committed, is archived as if history held it, and
nothing tells anyone. The archive is then the record of who approved the change.

Leif's choice on 2026-11-13 at about 04:04Z, "Every committed version": finalize, ship and archive
refuse a change whose `approvals.json` is uncommitted or differs from `HEAD`, or where any committed
version fails to extend the one before it. He approved the definition built from that choice as written
on 2026-11-19 at about 14:17Z, relayed by his planning session (hc t4c) through ss-lead (hc t58): the
frozen copy is `ss-lead/gates/428-definition.md`.

What already exists: the #660 check compares a reopened change's ledger with the one it archived
(`ledger_succession`, `relay_blind_approval_ledger_bytes`). This change adds the committed versions in
between, and the working copy against `HEAD`, on the three verbs that archive.

An unfinished branch, `feat/428-refuse-edited-approvals` (failing tests `7f15b21a`, a partial fix
`62d3cdc4`), covers finalize only and advises a restore in every case. Its tests are the start of this
change's tests.

## From the change's design.md

# Design

## One check, asked in four places

`src/change/historical_definition_digest_matches.rs`, beside finalize and archive, gains one read-only
check, `approvals_ledger_held_by_history(root, record)`. It reads; it never writes.

1. **The working copy against `HEAD`.** The ledger must be committed at `HEAD`. The working copy must
   equal that version byte for byte, with one exception: an `accepted` change's working copy may
   append, after the committed approvals and with nothing else changed, only the closing records its
   accepting step wrote: one `finalization` approval, or an `acceptance` approval with, before it, the
   `definition` approval `change accept` writes with the note "Normalized compatible definition
   evidence during explicit acceptance".
2. **Every committed version.** `git log --first-parent --no-renames` lists the commits that touched the
   file on `HEAD`'s first-parent history, bounded as other history reads are. Oldest first, each
   committed version must extend the last one before it. A commit at which the file does not exist (an
   archive move, a deletion) is skipped, so the next version is compared with the last one before it.

**Extends** means: for each of `approvals`, `reopenings` and `send_backs`, every record of the older
version is still at its position in the newer one and equal to it. Both versions are first read through
`relay_blind_approval_ledger_bytes`, the projection the #660 check uses, so the `relayed_by` of every
approval and of every reopening's `superseded_approval` is not compared. In a reopening, the
`stale_acceptance_input_digest` and `current_acceptance_input_digest` members may be added where the
older version lacked them, never changed. Other members of the ledger are not compared.

| Asked by | Where | State |
|---|---|---|
| `change finalize` | before `accept_change_with_gate` writes; on a resume, while the workspace is active | verifying, accepted |
| `change archive` | after the "archive after merge" refusal, before the first write | accepted |
| `change ship` | through finalize, and before it through the readiness check | verifying |
| `change ship-status`, `change ship --dry-run` | one blocker in `ship_status_report` | verifying, accepted |

## The ways out

The refusal says what fixes it, and which one depends on what differs:

- **Not committed at `HEAD`, or the working copy only adds records** (as after `change reopen` and a
  plain `change check`): commit it; `specsync change check <id> --commit` does.
- **The working copy changes or drops a committed record**: `git checkout HEAD -- <path>`.
- **A committed version rewrites an earlier one**: the refusal names the commit that rewrote it, the
  commit it rewrote and the record, such as `approvals[0]`, and gives the two ways out: take the
  rewriting commit out of the branch's history, or make the change again under a new id. When the file
  was removed in between, as when a workspace is deleted and created again under the same id, it says
  that too.

"Only adds records" is the same extends rule, applied to `HEAD` and the working copy.

## ship-status

`ship_status_report` asks the check for a verifying or accepted change and pushes its message as a
blocker, so `ready_to_finalize` is false. `change ship` and `change ship --dry-run` already refuse with
the report's first blocker, and ship-status exits 1 on a blocker (#654, REQ-cmd-change-039).

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
