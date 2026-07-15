---
change: CHG-0046-make-lifecycle-verification-workflows-evaluate-the-exact-pull-request-head-while
artifact: context
---

# Context

GitHub Actions checks pull requests out at a synthetic merge commit by default. PR #387
therefore ran lifecycle verification at merge ref `0c9d9e8` even though the persisted
verification evidence was correctly bound to exact PR head `03aa191`. The merge commit's
first-parent edge contains the substantive pull-request diff relative to `main`, so CHG45's
fail-closed every-parent freshness predicate correctly classified that checkout as stale.

This is an integration-boundary mismatch: lifecycle evidence must be evaluated at the exact
head it records, while ordinary build and test lanes should retain the synthetic merge so they
continue detecting integration failures against the current base branch.
