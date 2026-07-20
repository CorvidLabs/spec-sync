---
change: CHG-0061-document-the-5-2-0-lifecycle-surfaces-migrate-5-0-ledger-backfill-batch-and-si
artifact: context
---

# Context

SpecSync 5.2.0 shipped user-facing lifecycle surfaces that the documentation predates:
`migrate 5.0` ledger backfill (#404), single and batch `change correct-owner` (#403), `change
supersede`, squash-merged archival trust (#400), inert 5.0.1 registry stub tolerance (#405),
and the actionable staleness/migrate diagnostics (#404, #396). The CLI reference (`cli.md`)
documents none of them, `cross-project-refs.md` misses the registry tolerance, `workflow.md`
misses the archival trust model, and the `AGENTS.md` quick reference predates the SDD lifecycle
commands entirely. Documentation-only change; no canonical spec content or code changes.
