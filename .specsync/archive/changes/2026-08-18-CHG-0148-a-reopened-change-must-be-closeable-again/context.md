---
change: CHG-0148-a-reopened-change-must-be-closeable-again
artifact: context
---

# Context

`reopen` exists to recover a change whose accepted evidence went stale. It was
a dead end.

    change reopen <id>          ok
    change check <id> --commit  ok
    change review <id>          ok
    change finalize <id>        error: archived evidence failed post-move
                                validation; source restored: scoped review
                                history moved evidence outside finalization

The change was left in `accepted` with every route forward refusing, while
`change audit` reported "audit passed" over it and `ship-status` showed
product_tip and review_tip done. The recovery tool terminated in a state
strictly worse than the one it was invoked from: the change had been archived
before, and was now permanently active.

## Cause

`validate_scoped_review_history_transition` walks committed history comparing
consecutive appearances of the scoped-review ledger. When the CONTENT is
unchanged but the PATH moved, exactly one direction was allowed:

    previous_path.contains(".specsync/changes/")
        && current_path.contains(".specsync/archive/changes/")

That is the finalize move, active -> archive. `reopen` carries the same bytes
back the other way, archive -> active, and no branch admitted it.

## Why the refusal surfaced at finalize rather than at reopen

The rule is enforced by a WALK over committed history, not by the command
performing the move. `reopen` commits the archive -> active move and returns 0.
The next `finalize` re-walks, reaches the reopen commit, sees an unchanged
ledger at a path that moved in an unrecognised direction, and refuses.

That also explains why re-running `review` could never clear it — the offending
transition was already committed — and why readiness and refusal disagreed:
`ship-status` reads current state, the refusal reads history.
