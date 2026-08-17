---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: requirements
---

# Requirements

Two requirements, added as semantic deltas. The delta files are the source; `specs/` is
materialized from them.

## `deltas/change.md` — REQ-change-070

The domain rule: a lifecycle commit may not record a ledger below the highest already
committed, and must disclose any raise.

Stated as "no lifecycle commit" rather than "`check --commit`" deliberately. The report named
one command; three staging sites exist. A requirement naming the command would be satisfied by
a fix that left the other two able to commit the same regression.

## `deltas/cmd_change.md` — REQ-cmd-change-012

The command-surface rule: apply the floor before staging, and do not block the author.

Separate because "do not block" is a decision about the operator, not about the data. The
domain requirement would be equally satisfied by refusing the commit; this one records that
refusing is the wrong answer and why.

## `deltas/change.md` — REQ-change-071

The detection rule, added after gate 051 stayed red. Validation must refuse a ledger below the
mark the default branch published, *whether or not* the higher workspaces are on disk.

Separate from REQ-change-070 because they guard different directions: 070 stops a regression
being created, 071 stops one being accepted. A repository can receive a regressed ledger from
any clone, so closing only the write path would leave the audit still blessing it.

Its final criterion — read the published mark from the source the allocation floor already
uses — exists because a second implementation of the same lookup is how this codebase has
produced eight sibling-site defects.

## Explicitly retained behaviour

Three criteria exist to stop the fix over-reaching:

- A ledger at or above the committed mark is left exactly as written.
- Equal marks are not reported as a divergence.
- Acknowledged collisions from both sides survive the raise.

The first is the load-bearing control. Without it, "always restore the committed ledger" would
satisfy the headline requirement while destroying every new change's claim and reissuing IDs
that are already taken.
