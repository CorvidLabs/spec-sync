---
change: CHG-0014-route-archive-only-lifecycle-moves-through-a-minimal-specsync-ci-gate-while-pres
artifact: context
---

# Context

Archive-only PR #346 changed no executable product input, but the monolithic CI
workflow still ran platform tests, coverage, dependency audit, site, extension,
Action consumer, and CodeQL. Repository CI currently selects the whole workflow
for any `.specsync/**` path and has no job-level change classification.

The workflow must remain fail-closed. A change inside an active
`.specsync/changes/**` workspace is not an archive move and must not receive
the archive-only fast path.
