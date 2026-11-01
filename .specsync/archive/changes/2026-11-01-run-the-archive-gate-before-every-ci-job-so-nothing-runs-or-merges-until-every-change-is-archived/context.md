---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: context
---

# Context

Every SDD change must be archived before its pull request merges. CI already had an `Archive`
job and the required gate needed it, but it ran beside everything else on purpose: the old ship
flow pushed a product tip with the change still open, waited for tests and `SpecSync
implementation ready`, then pushed review/archive tips that reused the parent's green run. The
owner (0xLeif) asked for archival to block CI before anything runs, then to roll the same gate
out to every CorvidLabs repository that uses spec-sync (Raven, Chough and the rest).

Ruled out: keeping product-tip CI while also gating (it still runs CI before archival), and
gating only other repositories (spec-sync would teach a flow it does not follow itself).

A mid-flight session needs to know: the archived tip now carries all CI evidence, so local
`change check --commit` and `fledge lanes run verify` are the product verification before ship.
Published `@v6.1.1` predates the new Action inputs and ignores them, so other repositories
must pin a ref that contains this change.
