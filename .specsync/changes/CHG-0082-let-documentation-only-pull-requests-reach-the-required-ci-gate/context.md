---
change: CHG-0082-let-documentation-only-pull-requests-reach-the-required-ci-gate
artifact: context
---

# context

`Required CI gate` became a required status check, which was correct — without it
the gates were advisory and three pull requests merged red. But it means every
mergeable path must be able to reach the gate, and `docs/**` could not: the CI
workflow's path filter excluded it, so a documentation-only pull request never
triggered CI and the gate sat "Expected — waiting for status to be reported"
forever. PR #504 was the first to hit it.

Tried and rejected: teaching `classify` a documentation bucket so docs pull
requests skip the matrix. Cheaper, but path classification is load-bearing for
archive and review detection, and a wrong bucket there is worse than a slow docs
build. Deferred with its own fixture.
