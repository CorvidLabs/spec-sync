---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: context
---

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
