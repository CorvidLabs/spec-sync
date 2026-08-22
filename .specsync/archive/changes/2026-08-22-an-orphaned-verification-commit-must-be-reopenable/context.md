---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: context
---

# Context

Reported from a full day driving spec-sync on `CorvidLabs/site` — ~10 changes under 5.2.0, then
adopting workflow-v2 on 6.0.0. Filed as #673.

A rebase orphaned a change's verification commit. The CLI then had no forward verb:

    check:  accepted change verification commit is not in current history and canonical
            acceptance is not recorded on the remote default branch
    reopen: delivery inputs are current (exact or successor-covered); reopen is allowed
            only when delivery evidence is stale

Both statements are true at the same time. The content genuinely is current; it is the commit
that vanished. `check` refuses on reachability, `reopen` refuses on content, and reachability is
not a condition `reopen` could observe.

## The gate is a proxy, and it fails in both directions

The reporter's second observation is the one that settles the design. **An unrelated one-line
comment edit unlocks `reopen`.** So:

| situation | evidence actually good? | gate |
|---|---|---|
| rebase orphaned the verification commit | no — unreachable | refuses |
| unrelated comment edit to an input file | yes — nothing changed | permits |

The gate at `src/change.rs:3122` refuses the case it should admit and admits a case it has no
reason to. Its safety value was approximately zero — the escape hatch users find is "dirty a file
you did not want to dirty" — while its obstruction value was total. That is why this is a
correctness fix rather than a relaxation.

## Why widening reopen is not a widening of trust

Reopen was never gated on a capability an attacker lacks. Anyone who can rewrite local history to
orphan a commit can also edit one byte of a covered delivery file, drift the digest, and get a
reopen under the old rules. The gate is a staleness *classifier*, not an authorization check.

What actually authorizes is downstream and unchanged: `ledger_succession` (`src/change.rs:13236`)
still requires the ledger to contain the committed one verbatim with the new event superseding
exactly the terminal approval the committed generation closed on; `admissible_archive_introductions`,
`authenticated_accepted_transition_for`, and a fresh independent review all still apply. A control
test proves a tampered archive is still refused on this path.

## Already ruled out

- **Recomputing the anchor inside `reopened_change_preserves_sequence_history`.** That is a
  structural validator over recorded evidence; the anchor is a function of live git state that
  flips as history moves. It must read a recorded cause, not recompute.
- **Adding an anchor check to `validate_archived_integrity_inner`.** Archived records are inert
  history and are routinely unanchored on a squash-merged main — adding the check would turn
  essentially every existing archive red.
- **Message-only fix.** Explaining the deadlock without providing an exit leaves the dead end.
