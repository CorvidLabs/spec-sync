# Design

Two conjuncts replace one attacker-controlled count.

> **Who is speaking** — only the process writing a package out of the active workspace may
> present closing evidence history has not seen — **and what it says** — the ledger it is about
> to commit must contain, unrewritten, every ledger history already holds for this change.

Either alone is insufficient, and both prior candidate repairs failed by taking only one:
the first let any working tree speak for a committed package; the second let `finalize` bless a
package it merely found.

## `ledger_succession` replaces `generation`

`ArchiveIntroduction` carries the committed `approvals.json` bytes rather than a count. A
candidate ledger succeeds an earlier one only when `reopenings` grows, `approvals` is at least as
long, **both prefixes are byte-identical**, and the first added reopen event's
`superseded_approval` equals the earlier ledger's terminal approval.

Compared as raw `serde_json::Value`, never round-tripped through `ApprovalLedger` — the typed
form drops unknown fields, which is precisely where a difference would hide.

`reopenings.len()` is used nowhere in the decision.

## Cost

The index already ran one `git show` per introduction; it now keeps those bytes instead of
reducing them to a number. No additional git invocation.

## Measured

| | |
|---|---|
| suite | 2346 unit + 405 integration, 0 failed |
| drill 049 | `12/0/0` (was `11/1/1`) |
| drill 013 | PASSED (was FAIL) |
| drill 069 | `5/0` — unchanged, still refuses all three vectors |
| board | 41/17 → 51/7 |
| corpus | anchor repair byte-identical to HEAD; the scoped-review path set moves 6 archives ERR→OK, each verified append-only, none OK→ERR |

Strictly safer than the binary it repairs: the forged-generation variants of v4, v5 and v6 are
refused, and so is the front-door channel where `reopen` mints a genuine generation and a
carried-over approval actor is rewritten.
