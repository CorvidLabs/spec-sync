---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: design
---

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
the report's first blocker. Exit 1 on a blocker is #654 (PR #1093); this branch adds the assertion
when it is updated against main after #1093 merges (an open task until then).
