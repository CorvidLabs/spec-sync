---
id: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
state: implementing
type: feature
base_commit: cf28b4c5052afb72ce01454a353d2e99c83b3222
---

# Finalize, ship and archive refuse an approvals ledger that Git history does not hold

## Intent

finalize, ship and archive refuse an approvals ledger that Git history does not hold

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- `change finalize`, `change ship` and `change archive` refuse, writing nothing, a change whose `approvals.json` is not committed at `HEAD` or differs from the committed version, naming the file and the way out: the commit when the working copy only adds records, the restore when it changes a committed one.
- They refuse when a committed version of the ledger rewrites or drops an earlier approval, reopening or send-back, naming the commit that did it, the commit before it and the record; that covers a change deleted and created again under the same id, and a record an older SpecSync rewrote without a member it did not know.
- Every rewrite the tool itself makes still finalizes: a reopen, a send-back and fresh approval, a relay restored or removed by `change repair-relays` in an approval or in the approval a reopening embeds, the reopening members `specsync migrate 5.0` backfills, and a ledger member this version does not know.
- An accepted change archives with the closing records its accepting step wrote and has not committed, and no other uncommitted approval.
- `change ship-status` lists the refusal as a blocker and does not report the change ready, and `change ship --dry-run` refuses with it.
- The usual flow, approve, `check --commit`, review and finalize, archives as before.

## No-spec Rationale

Not applicable
