---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: design
---

# Design

The gate is `.github/scripts/archive-readiness.sh`, unchanged: it needs only Python and the
checkout, so it runs before any download or build. The Action runs it from
`${{ github.action_path }}` as its first step; `archive-gate-only` skips the download, check and
comment steps. CI's `archive` job calls the Action with `uses: ./`, so CI and every consumer run
the same gate.

Job graph: `archive` → `classify`/`preflight` → everything else; `implementation-gate` and
`ci-gate` run with `always()` and fail when `archive` did not succeed. A skipped required check
counts as passing on GitHub, so consumers must require an aggregate `always()` job, never a job
the gate can skip.

Classification uses the whole-PR diff only. With no open change reaching classify, tip-only
narrowing could only skip the product lane on the archived tip, which is now the only tip CI
tests. `review_required` can no longer be true past the gate, so corvid-pet selects on `full`.

Ship guidance keeps every JSON field and stage id; only text changes, and check-runs are read
on HEAD first because the archived tip is where CI decides the merge.
