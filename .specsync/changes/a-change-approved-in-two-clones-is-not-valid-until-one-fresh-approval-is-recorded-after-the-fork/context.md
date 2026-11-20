---
change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
artifact: context
---

# Context

- **#532** (area:security, transferred from sandbox #11): two clones approve the same change with different actors. Both `change approve` runs exit 0, because the project lock covers one working tree. Merging leaves two approvals ledgers in history that neither extends.
- **Reproduced 2026-11-20 through the binary** (`ss-builder/evidence/532-repro/run.sh`): two real clones of one origin, and `git merge -X ours`. On released 6.3.0 and on #428's branch (9b75e8ba), `change status`, `change check` and `change audit --strict` exit 0. Clone B's approval is in history but not at HEAD.
- **Why #428 misses it:** #428 walks the ledger's first-parent history only, so a merge that keeps the first parent's ledger extends it.
- **Leif's decisions:** 2026-11-13, "build it" and "One fresh approval (Recommended)"; 2026-11-20 about 15:06Z, "Approve (Recommended)" for this definition and its four recommended choices. All relayed by hc t4c.
- **Prior attempt:** the unapproved 2026-11-13 branch `feat/532-forked-approvals` (56faf601). Its failing tests and its walk are kept.
