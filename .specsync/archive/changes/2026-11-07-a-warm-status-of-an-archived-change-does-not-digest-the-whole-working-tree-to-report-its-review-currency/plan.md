---
change: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency
artifact: plan
---

# Plan

## Approach

For an archived change, `summarize_change_with_effective` takes the project-input digest for `scoped_review_current` only while the archived package has uncommitted changes, the window between `finalize` and its commit. A committed archive reports `false` without the digest, which is what the full check gives in every committed case pinned by `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives`. States before archive are unchanged.

Alternative for Leif (A): report `false` for every archived change. That is simpler, but it changes one observed output: `true` right after a standalone `finalize`, before its commit. `ship` commits at once, so the window is short there.

## Out of scope

- `terminal_evidence_summary` on cached archives, the remaining 33% of the warm time; it is profiled next.
- Chough's `snapshot.hash` phase (1.1 s or more), which is Chough-side.
- Any change to the project-input digest itself, or to review currency before archive.

## Steps

1. Failing guard committed (`a5b54557`); equivalence pin committed.
2. Gate the digest for archived changes on the package having uncommitted changes.
3. The guard turns green; the pin and all warm, cold and lifecycle tests pass unchanged.
4. Raven warm before and after; CHANGELOG; `change check --commit`, verify, `change audit --strict`.

## Risks

- **Committed means false is pinned, not proven.** If some committed archive could still match its review's digest, it would report `false` where today it reports `true`. Nothing in the next action or gates reads the field for an archived change (only `Verifying` uses it), and Chough's source doesn't read it. The pin covers the shapes found so far.
- **The uncommitted check must be cheap.** It uses the package's own paths (`scoped_edits_uncommitted`-style `diff-files`/`ls-files` on the package), never the whole tree.

## Constraints consulted

- SPEED-1.a, SPEED-1.d.
- The summary already computes `verification_current` and `scoped_edits_uncommitted` only in the states that use them.
- change.3md "Finalize judges only what it archives": read paths keep live-identical answers.
- REQ-change-119: the archived-read guards stay.
